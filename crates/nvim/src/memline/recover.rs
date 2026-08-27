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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::allocator::Owned;
use crate::buffer::{BufFlags, alloc_unregistered_buffer};
use crate::guard::Suppress;
use crate::{semsg_c, smsg_c};
use core::ffi::{c_char, c_int, c_long, c_uint};

use super::*;
use crate::highlight_group::HLF_E;
use crate::types::{FAIL, MAXPATHL, NUL, OK, OptionSetFlags};
use crate::winlayer::{Buf, Win};

/// Try to recover `curbuf` from its swap file.
///
/// `checkext`: whether the buffer's own name may itself be a swap file name,
/// as it is for `nvim -r file.swp`.
/// Read the original file into `curbuf`, starting after line `from`.
///
/// Both callers name `curbuf`'s own full path and pass no `:read` command,
/// which is the whole of `readfile`'s precondition; they differ only in the
/// range and in whether the buffer is new.
fn read_original(from: linenr_T, skip: linenr_T, lines: linenr_T, flags: c_int) -> c_int {
    let (name, short) = (cur_buf().b_ffname, core::ptr::null_mut());
    let no_cmd = core::ptr::null_mut();
    // SAFETY: the name is the buffer's own, and a null `eap` is "no command".
    unsafe { readfile(name, short, from, skip, lines, no_cmd, flags, false) }
}

pub unsafe fn ml_recover(checkext: bool) {
    // The recovery report runs autocommands between the calls that fill this,
    // so it is this frame's rather than the shared `NameBuff`.
    let mut path = [0 as c_char; MAXPATHL as usize];
    recoverymode.set(true);
    let called_from_main = cur_buf().b_ml.ml_mfp.is_null();

    let mut buf: *mut buf_T = core::ptr::null_mut();
    // Who owns what `buf` points at. The recovery buffer is not in the
    // registry, so this frame is its owner; `buf` is only the address the
    // memline code below works through.
    let mut owned_buf: Option<Owned<buf_T>> = None;
    let mut mfp: *mut memfile_T = core::ptr::null_mut();
    let mut hp: *mut bhdr_T = core::ptr::null_mut();
    let mut fname_used: *mut c_char = core::ptr::null_mut();
    // Nothing was recovered yet, so a failure now leaves the buffer with
    // no memline at all.
    let mut serious_error = true;

    'theend: {
        let fname = if cur_buf().b_fname.is_null() {
            c"".as_ptr().cast_mut()
        } else {
            cur_buf().b_fname
        };
        // A name ending in ".s[a-w][a-z]" is taken to be the swap file
        // itself; otherwise its swap files are searched for.
        let directly = checkext && unsafe { looks_like_swapfile(fname) };
        fname_used = if directly {
            unsafe { xstrdup(fname) } // a copy for mf_open(), which consumes it
        } else {
            let Some(chosen) = (unsafe { choose_swapfile(fname) }) else {
                break 'theend;
            };
            chosen
        };
        if fname_used.is_null() {
            break 'theend; // the user chose an invalid number
        }
        // When called from main() the storage structure still needs
        // initialising.
        if called_from_main && unsafe { ml_open(curbuf.get()) } == FAIL {
            unsafe { getout(1) };
        }

        // A buffer structure for the swap file being recovered. Only the
        // memline in it is really used, and it is never registered or
        // put on the buffer list -- see `alloc_unregistered_buffer`.
        buf = owned_buf.insert(alloc_unregistered_buffer()).address();
        unsafe { (*buf).b_ml.stack_clear() }; // nothing in the stack
        unsafe { (*buf).b_ml.clear_cache() }; // no cached line
        unsafe { (*buf).b_ml.ml_locked = None }; // no locked block
        unsafe { (*buf).b_ml.ml_flags = MlFlags::NONE };

        // Open the memfile on the old swap file. `mf_open` consumes the
        // name, so keep a copy of it for the messages.
        let kept = unsafe { xstrdup(fname_used) };
        mfp = unsafe { mf_open(fname_used, O_RDONLY) };
        fname_used = kept;
        if mfp.is_null() || unsafe { (*mfp).mf_fd } < 0 {
            unsafe { semsg_c!(tr(c"E306: Cannot open %s"), fname_used) };
            break 'theend;
        }
        unsafe { (*buf).b_ml.ml_mfp = mfp };

        // The page size `mf_open` picked need not be the one the swap
        // file was written with; the real one is in block zero. Reading
        // block zero needs *a* page size, so use the smallest one a swap
        // file can have, and correct it below.
        unsafe { (*mfp).mf_page_size = MIN_SWAP_PAGE_SIZE };

        let hl_id = HLF_E;
        unsafe { msg_ext_set_kind(c"emsg".as_ptr()) };
        hp = unsafe { mf_get(mfp, 0, 1) };
        if hp.is_null() {
            unsafe { msg_start() };
            note(c"Unable to read block 0 from ", hl_id);
            unsafe { msg_outtrans(mf_fname(mfp), hl_id, true) };
            note(
                c"\nMaybe no changes were made or Nvim did not update the swap file.",
                hl_id,
            );
            unsafe { msg_end() };
            break 'theend;
        }
        let mut b0p = unsafe { (*hp).bh_data } as *mut ZeroBlock;
        if unsafe { strncmp((*b0p).b0_version.as_ptr(), c"VIM 3.0".as_ptr(), 7) } == 0 {
            unsafe { msg_start() };
            unsafe { msg_outtrans(mf_fname(mfp), 0, true) };
            note(c" cannot be used with this version of Nvim.\n", 0);
            note(c"Use Vim version 3.0.\n", 0);
            unsafe { msg_end() };
            break 'theend;
        }
        if !ml_check_b0_id(unsafe { &*b0p }) {
            unsafe {
                semsg_c!(
                    tr(c"E307: %s does not look like a Nvim swap file"),
                    mf_fname(mfp),
                )
            };
            break 'theend;
        }
        if b0_magic_wrong(unsafe { &*b0p }) {
            unsafe { msg_start() };
            unsafe { msg_outtrans(mf_fname(mfp), hl_id, true) };
            note(c" cannot be used on this computer.\n", hl_id);
            note(c"The file was created on ", hl_id);
            // Terminate the name field, so that printing the host name
            // cannot run off the end of a corrupted one.
            unsafe { (*b0p).b0_fname[0] = NUL as c_char };
            unsafe { msg_puts_hl((*b0p).b0_hname.as_ptr(), hl_id, true) };
            note(c",\nor the file has been damaged.", hl_id);
            unsafe { msg_end() };
            break 'theend;
        }

        // The guessed page size was wrong, so the highest block number
        // in the file has to be worked out again.
        let recorded_page_size = unsafe { b0_read_number(&(*b0p).b0_page_size) } as c_uint;
        if unsafe { (*mfp).mf_page_size } != recorded_page_size {
            let previous_page_size = unsafe { (*mfp).mf_page_size };
            unsafe { mf_new_page_size(mfp, recorded_page_size) };
            if unsafe { (*mfp).mf_page_size } < previous_page_size {
                unsafe { msg_start() };
                unsafe { msg_outtrans(mf_fname(mfp), hl_id, true) };
                note(
                    c" has been damaged (page size is smaller than minimum value).\n",
                    hl_id,
                );
                unsafe { msg_end() };
                break 'theend;
            }
            let size = unsafe { lseek((*mfp).mf_fd, 0, SEEK_END) };
            // Zero means no file, or an empty one.
            unsafe {
                (*mfp).mf_blocknr_max = if size <= 0 {
                    0
                } else {
                    size / (*mfp).mf_page_size as off_T
                } as blocknr_T
            };
            unsafe { (*mfp).mf_infile_count = (*mfp).mf_blocknr_max };

            // Block zero's own buffer was allocated at the guessed size.
            let bigger = unsafe { xmalloc((*mfp).mf_page_size as size_t) };
            unsafe { memmove(bigger, (*hp).bh_data, previous_page_size as size_t) };
            unsafe { xfree((*hp).bh_data) };
            unsafe { (*hp).bh_data = bigger };
            b0p = bigger as *mut ZeroBlock;
        }

        // Given the swap file's name directly, the buffer takes its name
        // from what the swap file says it belongs to.
        if directly {
            unsafe { expand_env((*b0p).b0_fname.as_mut_ptr(), path.as_mut_ptr(), MAXPATHL) };
            if unsafe { setfname(cur_buf(), path.as_mut_ptr(), core::ptr::null_mut(), true) }
                == FAIL
            {
                break 'theend;
            }
        }

        unsafe { msg_ext_set_kind(c"wmsg".as_ptr()) };
        msg_ext_skip_flush.set(true);
        let (out, room) = (path.as_mut_ptr(), MAXPATHL as size_t);
        let none = core::ptr::null();
        unsafe { home_replace(none, mf_fname(mfp), out, room, true) };
        unsafe { smsg_c!(0, tr(c"Using swap file \"%s\""), path.as_ptr()) };
        if !unsafe { buf_spname(curbuf.get()) }.is_null() {
            unsafe {
                xstrlcpy(
                    path.as_mut_ptr(),
                    buf_spname(curbuf.get()),
                    MAXPATHL as size_t,
                )
            };
        } else {
            let (out, room) = (path.as_mut_ptr(), MAXPATHL as size_t);
            let none = core::ptr::null();
            unsafe { home_replace(none, cur_buf().b_ffname, out, room, true) };
        }
        unsafe { msg_putchar('\n' as c_int) };
        unsafe { smsg_c!(0, tr(c"Original file \"%s\""), path.as_ptr()) };
        unsafe { msg_putchar('\n' as c_int) };
        msg_ext_skip_flush.set(false);

        // Compare the dates of the swap file and the original.
        let mtime = unsafe { b0_read_number(&(*b0p).b0_mtime) } as c_int;
        let mut org_file_info: FileInfo = unsafe { core::mem::zeroed() };
        let mut swp_file_info: FileInfo = unsafe { core::mem::zeroed() };
        if !cur_buf().b_ffname.is_null()
            && unsafe { os_fileinfo(cur_buf().b_ffname, &raw mut org_file_info) }
            && ((unsafe { os_fileinfo(mf_fname(mfp), &raw mut swp_file_info) }
                && org_file_info.stat.st_mtim.tv_sec > swp_file_info.stat.st_mtim.tv_sec)
                || org_file_info.stat.st_mtim.tv_sec != mtime as _)
        {
            complain(c"E308: Warning: Original file may have been changed");
        }
        unsafe { ui_flush() };

        // Take 'fileformat' and 'fileencoding' from block zero. The
        // encoding sits at the very end of the name field, behind a NUL,
        // so it is found by scanning back from there.
        let b0_ff = unsafe { (*b0p).flags() } & B0_FF_MASK;
        let mut b0_fenc: *mut c_char = core::ptr::null_mut();
        if unsafe { (*b0p).flags() } & B0_HAS_FENC != 0 {
            let name = unsafe { (*b0p).b0_fname.as_mut_ptr() };
            let end = unsafe { name.offset(B0_FNAME_SIZE_NOCRYPT as isize) };
            let mut p = end;
            while p > name && unsafe { *p.offset(-1) } as c_int != NUL {
                p = unsafe { p.offset(-1) };
            }
            b0_fenc = unsafe { xstrnsave(p, end.offset_from(p) as size_t) };
        }

        // Release block zero. `b0p` is still read further down, for the
        // "process STILL RUNNING" note: `mf_put` only drops the block's
        // reference, and nothing here can make the memfile hand its page
        // back out.
        unsafe { mf_put(mfp, hp, false, false) };
        hp = core::ptr::null_mut();

        // Recovery is going ahead, so the buffer's current contents go.
        while !cur_buf().b_ml.ml_flags.has(MlFlags::EMPTY) {
            unsafe { ml_delete(1) };
        }

        // Read the original file, to pick up 'fileformat', 'fileencoding'
        // and friends. Errors are ignored, and the text itself is not
        // used — except as the "unchanged?" comparison below.
        let mut orig_file_status = NOTDONE;
        if !cur_buf().b_ffname.is_null() {
            orig_file_status = read_original(0, 0, MAXLNUM as linenr_T, READ_NEW as c_int);
        }

        // What the swap file recorded wins over what the file suggests.
        if b0_ff != 0 {
            set_fileformat(b0_ff - 1, OptionSetFlags::LOCAL);
        }
        if !b0_fenc.is_null() {
            set_option_value_give_err(
                kOptFileencoding,
                OptVal {
                    type_0: kOptValTypeString,
                    data: OptValData {
                        string: unsafe { cstr_as_string(b0_fenc) },
                    },
                },
                OptionSetFlags::LOCAL,
            );
            unsafe { xfree(b0_fenc.cast()) };
        }
        unchanged(cur_buf(), true, true);

        serious_error = false;
        let Ok((lnum, error)) = (unsafe { recover_lines(buf, mfp, &mut hp) }) else {
            break 'theend;
        };

        // Compare the recovered contents with the original file's.
        // Lines 1 to lnum are what was recovered, lines lnum + 1 to
        // ml_line_count are the file's, and line ml_line_count + 1 is the
        // empty buffer's dummy line.
        if orig_file_status != OK || cur_buf().b_ml.ml_line_count != lnum * 2 + 1 {
            // Recovering an empty file gives two lines of which the first
            // is empty; that is not a modification.
            if !(cur_buf().b_ml.ml_line_count == 2 && unsafe { *ml_get(1) } as c_int == NUL) {
                changed_internal(cur_buf());
                unsafe { buf_inc_changedtick(curbuf.get()) };
            }
        } else {
            for idx in 1..=lnum {
                // One of the two lines has to be copied: fetching the
                // other may flush it.
                let p = unsafe { xstrnsave(ml_get(idx), ml_get_len(idx) as size_t) };
                let same = unsafe { strcmp(p, ml_get(idx + lnum)) } == 0;
                unsafe { xfree(p.cast()) };
                if !same {
                    changed_internal(cur_buf());
                    unsafe { buf_inc_changedtick(curbuf.get()) };
                    break;
                }
            }
        }

        // Drop the original file's lines and the empty buffer's dummy
        // line; they are now past the end of what was recovered.
        while cur_buf().b_ml.ml_line_count > lnum && !cur_buf().b_ml.ml_flags.has(MlFlags::EMPTY) {
            unsafe { ml_delete(cur_buf().b_ml.ml_line_count) };
        }
        cur_buf().b_flags |= BufFlags::RECOVERED;
        check_cursor(unsafe { Win::current() });

        msg_ext_skip_flush.set(!got_int.get());
        recoverymode.set(false);
        unsafe { report_recovery(error, b0p, fname_used) };
        redraw_curbuf_later(UPD_NOT_VALID);
    }

    msg_ext_skip_flush.set(false);
    unsafe { xfree(fname_used.cast()) };
    recoverymode.set(false);
    if !mfp.is_null() {
        if !hp.is_null() {
            unsafe { mf_put(mfp, hp, false, false) };
        }
        unsafe { mf_close(mfp, false) }; // also frees the swap file's name
    }
    // The free: `buf_T`'s destructor runs, taking the block stack with
    // it, and the memory goes back.
    drop(owned_buf);
    if serious_error && called_from_main {
        unsafe { ml_close(curbuf.get(), 1) };
    } else {
        let (name, buf) = (cur_buf().b_fname, curbuf.get());
        let none = core::ptr::null_mut();
        unsafe { apply_autocmds(EVENT_BUFREADPOST, none, name, false, buf) };
        unsafe { apply_autocmds(EVENT_BUFWINENTER, none, name, false, buf) };
    }
}

/// Whether this name is itself a swap file name: it ends in `.s`, a letter
/// from `a` to `w`, and any letter — the extensions `findswapname` permutes
/// through.
unsafe fn looks_like_swapfile(fname: *mut c_char) -> bool {
    let len = unsafe { strlen(fname) } as isize;
    len >= 4
        && unsafe { strncasecmp(fname.offset(len - 4), c".s".as_ptr(), 2) } == 0
        && !unsafe {
            vim_strchr(
                c"abcdefghijklmnopqrstuvw".as_ptr(),
                (*fname.offset(len - 2) as u8).to_ascii_lowercase() as c_int,
            )
        }
        .is_null()
        && (unsafe { *fname.offset(len - 1) } as u8).is_ascii_alphabetic()
}

/// Pick which of `fname`'s swap files to recover from: the only one there is,
/// or the one the user names out of a listing.
///
/// Returns the allocated name, or `None` to give up.
unsafe fn choose_swapfile(fname: *mut c_char) -> Option<*mut c_char> {
    let (dir, out) = (core::ptr::null_mut(), core::ptr::null_mut());
    let count = unsafe { recover_names(fname, false, dir, 0, out) };
    if count == 0 {
        unsafe { semsg_c!(tr(c"E305: No swap file found for %s"), fname) };
        return None;
    }
    let nr = if count == 1 {
        1
    } else {
        unsafe { recover_names(fname, true, core::ptr::null_mut(), 0, core::ptr::null_mut()) };
        if !ui_has(kUIMessages) {
            unsafe { msg_putchar('\n' as c_int) };
        }
        let nr = unsafe {
            prompt_for_input(
                tr(c"Enter number of swap file to use (0 to quit): "),
                0,
                false,
                core::ptr::null_mut(),
            )
        };
        if nr < 1 || nr > count {
            return None;
        }
        nr
    };
    let mut fname_used: *mut c_char = core::ptr::null_mut();
    unsafe { recover_names(fname, false, core::ptr::null_mut(), nr, &raw mut fname_used) };
    Some(fname_used)
}

/// Walk the swap file's block tree and append every line it can find to
/// `curbuf`, after line 0.
///
/// `buf` is the scratch buffer whose `ml_stack` records the descent. Nothing
/// in the file is trusted: a count or a block number that cannot be right
/// costs a `???` line and the walk carries on. Returns the number of lines
/// appended and the number of problems found, or `Err` when block 1 itself is
/// unusable, which leaves nothing to recover.
unsafe fn recover_lines(
    buf: *mut buf_T,
    mfp: *mut memfile_T,
    hp: &mut *mut bhdr_T,
) -> Result<(linenr_T, c_int), ()> {
    let mut bnum: blocknr_T = 1; // start with block 1
    let mut page_count: c_uint = 1; // which is one page
    let mut lnum: linenr_T = 0; // append after line 0 in curbuf
    let mut line_count: linenr_T = 0;
    let mut idx = 0; // start with the first index in block 1
    let mut error = 0;
    unsafe { (*buf).b_ml.stack_clear() };

    // Without a file to fall back on, a data block whose number went
    // negative (never written to the swap file) is simply lost.
    let mut cannot_open = cur_buf().b_ffname.is_null();

    let append = |lnum: &mut linenr_T, text: *const c_char| {
        unsafe { ml_append(*lnum, text.cast_mut(), 0, true) };
        *lnum += 1;
    };

    'walk: while !got_int.get() {
        'step: {
            if !hp.is_null() {
                unsafe { mf_put(mfp, *hp, false, false) }; // release the previous block
            }
            *hp = unsafe { mf_get(mfp, bnum, page_count) };
            if hp.is_null() {
                if bnum == 1 {
                    unsafe {
                        semsg_c!(tr(c"E309: Unable to read block 1 from %s"), mf_fname(mfp),)
                    };
                    return Err(());
                }
                error += 1;
                append(&mut lnum, tr(c"???MANY LINES MISSING"));
            } else if unsafe { (*((**hp).bh_data as *mut PointerBlock)).pb_id }
                == PTR_ID as uint16_t
            {
                let pp = unsafe { (**hp).bh_data } as *mut PointerBlock;
                // The counts in the header have to fit the page size this
                // build uses, or the entries cannot be walked at all.
                let count_max = PointerBlock::count_max(unsafe { (*mfp).mf_page_size });
                let mut ptr_block_error = false;
                if unsafe { (*pp).pb_count_max } != count_max {
                    ptr_block_error = true;
                    unsafe { (*pp).pb_count_max = count_max };
                }
                if unsafe { (*pp).pb_count } > unsafe { (*pp).pb_count_max } {
                    ptr_block_error = true;
                    unsafe { (*pp).pb_count = (*pp).pb_count_max };
                }
                if ptr_block_error {
                    complain(c"E1364: Warning: Pointer block corrupted");
                }

                // The first time down this block, its entries should
                // account for exactly the line count promised above it.
                if idx == 0 && line_count != 0 {
                    for i in 0..unsafe { (*pp).pb_count } as usize {
                        let entry = pb_entries(pp).wrapping_add(i);
                        line_count -= unsafe { (*entry).pe_line_count };
                    }
                    if line_count != 0 {
                        error += 1;
                        append(&mut lnum, tr(c"???LINE COUNT WRONG"));
                    }
                }

                if unsafe { (*pp).pb_count } == 0 {
                    append(&mut lnum, tr(c"???EMPTY BLOCK"));
                    error += 1;
                } else if idx < unsafe { (*pp).pb_count } as c_int {
                    let pe = unsafe { *pb_entries(pp).wrapping_add(idx as usize) };
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
                                || read_original(lnum, pe.pe_old_lnum - 1, line_count, 0) != OK
                            {
                                cannot_open = true;
                            } else {
                                lnum += line_count;
                            }
                        }
                        if cannot_open {
                            error += 1;
                            append(&mut lnum, tr(c"???LINES MISSING"));
                        }
                        idx += 1; // same block again, for the next index
                        break 'step;
                    }

                    // One block deeper in the tree.
                    let top = unsafe { ml_add_stack(buf) };
                    let frame = infoptr_T {
                        ip_bnum: bnum,
                        ip_low: 0,
                        ip_high: 0,
                        ip_index: idx,
                    };
                    unsafe { (*buf).b_ml.stack_set(top, frame) };

                    bnum = pe.pe_bnum;
                    line_count = pe.pe_line_count;
                    page_count = pe.pe_page_count as c_uint;
                    // `pe_page_count` sizes the allocation `mf_get` makes,
                    // so a bogus value (0x40000000, say) would ask for
                    // gigabytes. It must be at least one page, and the
                    // block must lie inside the file.
                    if page_count < 1
                        || bnum + page_count as blocknr_T > unsafe { (*mfp).mf_blocknr_max } + 1
                    {
                        error += 1;
                        append(&mut lnum, tr(c"???ILLEGAL BLOCK NUMBER"));
                        // Skip this entry and pop back up, to recover
                        // whatever else there is.
                        let ip = unsafe { (*buf).b_ml.stack_at(top) };
                        idx = ip.ip_index + 1;
                        bnum = ip.ip_bnum;
                        page_count = 1;
                        unsafe { (*buf).b_ml.stack_pop() };
                        break 'step;
                    }
                    idx = 0;
                    break 'step;
                }
            } else {
                let dp = unsafe { (**hp).bh_data } as *mut DataBlock;
                if unsafe { (*dp).db_id } != DATA_ID as uint16_t {
                    if bnum == 1 {
                        unsafe {
                            semsg_c!(
                                tr(c"E310: Block 1 ID wrong (%s not a .swp file?)"),
                                mf_fname(mfp),
                            )
                        };
                        return Err(());
                    }
                    error += 1;
                    append(&mut lnum, tr(c"???BLOCK MISSING"));
                } else {
                    // A data block: append every line in it.
                    let mut has_error = false;

                    // The block's length has to match the page count the
                    // pointer block promised; if it does not, trust the
                    // pointer block.
                    if page_count.wrapping_mul(unsafe { (*mfp).mf_page_size })
                        != unsafe { (*dp).db_txt_end }
                    {
                        append(
                            &mut lnum,
                            tr(c"??? from here until ???END lines may be messed up"),
                        );
                        error += 1;
                        has_error = true;
                        let end = page_count.wrapping_mul(unsafe { (*mfp).mf_page_size });
                        unsafe { (*dp).db_txt_end = end };
                    }

                    // Make sure the block ends in a NUL, so that copying
                    // the last line out of it cannot run past the end.
                    unsafe { *db_byte(dp, (*dp).db_txt_end as isize - 1) = NUL as c_char };

                    // Likewise for the line count.
                    if line_count as c_long != unsafe { (*dp).db_line_count } {
                        append(
                            &mut lnum,
                            tr(c"??? from here until ???END lines may have been inserted/deleted"),
                        );
                        error += 1;
                        has_error = true;
                    }

                    let mut did_questions = false;
                    for i in 0..unsafe { (*dp).db_line_count } {
                        let index = unsafe {
                            (&raw mut (*dp).db_index)
                                .cast::<c_uint>()
                                .offset(i as isize)
                        };
                        if index as *mut c_char
                            >= unsafe { db_byte(dp, (*dp).db_txt_start as isize) }
                        {
                            // The line count must be wrong: the index
                            // array has run into the text.
                            error += 1;
                            append(&mut lnum, tr(c"??? lines may be missing"));
                            break;
                        }
                        let txt_start = (unsafe { index.read() } & DB_INDEX_MASK) as c_int;
                        let text = if txt_start <= HEADER_SIZE as c_int
                            || txt_start >= unsafe { (*dp).db_txt_end } as c_int
                        {
                            error += 1;
                            // One "???" for a run of them is enough.
                            if did_questions {
                                continue;
                            }
                            did_questions = true;
                            c"???".as_ptr()
                        } else {
                            did_questions = false;
                            db_byte(dp, txt_start as isize)
                        };
                        append(&mut lnum, text);
                    }
                    if has_error {
                        append(&mut lnum, tr(c"???END"));
                    }
                }
            }

            // One block back up the tree, and on to the next index.
            let Some(ip) = (unsafe { (*buf).b_ml.stack_pop() }) else {
                break 'walk; // finished
            };
            bnum = ip.ip_bnum;
            idx = ip.ip_index + 1;
            page_count = 1;
        }
        line_breakcheck();
    }

    Ok((lnum, error))
}

/// Say how it went, and what the user should do next.
unsafe fn report_recovery(error: c_int, b0p: *const ZeroBlock, fname_used: *const c_char) {
    if got_int.get() {
        complain(c"E311: Recovery Interrupted");
        return;
    }
    if error != 0 {
        let no_prompt = Suppress::wait_return();
        unsafe { msg_ext_set_kind(c"emsg".as_ptr()) };
        unsafe { msg(c">>>>>>>>>>>>>\n".as_ptr(), 0) };
        unsafe {
            emsg(tr(
                c"E312: Errors detected while recovering; look for lines starting with ???",
            ))
        };
        drop(no_prompt);
        unsafe { msg_putchar('\n' as c_int) };
        tell(c"See \":help E312\" for more information.", 0);
        unsafe { msg(c"\n>>>>>>>>>>>>>".as_ptr(), 0) };
        return;
    }

    unsafe { msg_ext_set_kind(c"wmsg".as_ptr()) };
    if cur_buf().b_changed != 0 {
        tell(
            c"Recovery completed. You should check if everything is OK.",
            0,
        );
        unsafe {
            msg_puts(tr(
                c"\n(You might want to write out this file under another name\n",
            ))
        };
        unsafe {
            msg_puts(tr(
                c"and run diff with the original file to check for changes)",
            ))
        };
    } else {
        tell(
            c"Recovery completed. Buffer contents equals file contents.",
            0,
        );
    }
    say(c"\nYou may want to delete the .swp file now.");
    if swapfile_proc_running(unsafe { &*b0p }, fname_used) != 0 {
        // There may be a live Nvim on the same file; the user may want to
        // go and kill it.
        say(c"\nNote: process STILL RUNNING: ");
        unsafe { msg_outnum((*b0p).pid() as c_int) };
    }
    if !ui_has(kUIMessages) {
        unsafe { msg_puts(c"\n\n".as_ptr()) };
    }
    cmdline_row.set(msg_row.get());
}

/// Sync every buffer's memline.
///
/// `check_file`: also check that the original file still exists and is
/// unchanged. `check_char`: stop as soon as a character is typed, having
/// synced at least one block.
pub unsafe fn ml_sync_all(check_file: c_int, check_char: c_int, do_fsync: bool) {
    for buf in buffers() {
        // SAFETY: a live buffer from the editor's own list, and the memfile
        // it owns.
        if !buf.b_ml.ml_mfp.is_null() && !unsafe { mf_fname(buf.b_ml.ml_mfp) }.is_null() {
            unsafe { ml_flush_line(buf.raw(), false) }; // flush the buffered line
            unsafe { ml_find_line(buf.raw(), 0, ML_FLUSH as c_int) }; // flush the locked block

            if buf_is_changed(buf)
                && check_file != 0
                && unsafe { mf_need_trans(buf.b_ml.ml_mfp) }
                && !buf.b_ffname.is_null()
            {
                // If the original file is gone or has changed, preserve
                // now, to get rid of all the negative numbered blocks.
                let mut file_info: FileInfo = unsafe { core::mem::zeroed() };
                if !unsafe { os_fileinfo(buf.b_ffname, &raw mut file_info) }
                    || file_info.stat.st_mtim.tv_sec != buf.b_mtime_read
                    || file_info.stat.st_mtim.tv_nsec != buf.b_mtime_read_ns
                    || unsafe { os_fileinfo_size(&raw mut file_info) } != buf.b_orig_size
                {
                    unsafe { ml_preserve(buf.raw(), false, do_fsync) };
                    did_check_timestamps.set(false);
                    need_check_timestamps.set(true); // give the message later
                }
            }

            if unsafe { (*buf.b_ml.ml_mfp).mf_dirty } == MfDirty::Yes {
                // Best effort: `ml_sync_all` writes what it can and
                // leaves the rest dirty.
                let stop = if check_char != 0 {
                    MFS_STOP as c_int
                } else {
                    0
                };
                let flush = if do_fsync && buf_is_changed(buf) {
                    MFS_FLUSH as c_int
                } else {
                    0
                };
                let _ = unsafe { mf_sync(buf.b_ml.ml_mfp, stop | flush) };
                if check_char != 0 && os_char_avail() {
                    break; // a character is available now
                }
            }
        }
    }
}

/// Sync one buffer, including its negative numbered blocks, so that
/// afterwards everything is in the swap file.
///
/// This is `:preserve`, and what happens when the original file has been
/// changed or deleted. `message` reports whether it worked.
pub unsafe fn ml_preserve(buf: *mut buf_T, message: bool, do_fsync: bool) {
    let mfp = unsafe { (*buf).b_ml.ml_mfp };
    if mfp.is_null() || unsafe { mf_fname(mfp) }.is_null() {
        if message {
            complain(c"E313: Cannot preserve, there is no swap file");
        }
        return;
    }

    // Only an interrupt from here on counts, not one from before.
    let got_int_save = got_int.get();
    got_int.set(false);

    unsafe { ml_flush_line(buf, false) }; // flush the buffered line
    unsafe { ml_find_line(buf, 0, ML_FLUSH as c_int) }; // flush the locked block
    let sync_flags = MFS_ALL as c_int | if do_fsync { MFS_FLUSH as c_int } else { 0 };
    // `ml_preserve` still answers OK/FAIL to its own callers, so the
    // memfile's result is converted here and again below.
    let mut status = unsafe { mf_sync(mfp, sync_flags) }.map_or(FAIL, |()| OK);
    unsafe { (*buf).b_ml.stack_clear() }; // the stack is invalid after MFS_ALL

    // Some data blocks may have gone from a negative to a positive block
    // number, which means the pointer blocks referring to them need
    // updating. Which pointer blocks those are is not recorded, so every
    // data block is visited until no translations are left (or the end of
    // the file is reached, which can only happen when a write failed —
    // a full file system, say). `ml_find_line` does the work, translating
    // the negative numbers as it fetches each block's first line.
    'theend: {
        if unsafe { mf_need_trans(mfp) } && !got_int.get() {
            let mut lnum: linenr_T = 1;
            while unsafe { mf_need_trans(mfp) } && lnum <= unsafe { (*buf).b_ml.ml_line_count } {
                if unsafe { ml_find_line(buf, lnum, ML_FIND as c_int) }.is_null() {
                    status = FAIL;
                    break 'theend;
                }
                lnum = unsafe { (*buf).b_ml.locked_high() } + 1;
            }
            unsafe { ml_find_line(buf, 0, ML_FLUSH as c_int) }; // flush the locked block
            // Sync the pointer blocks that were just updated.
            if unsafe { mf_sync(mfp, sync_flags) }.is_err() {
                status = FAIL;
            }
            unsafe { (*buf).b_ml.stack_clear() }; // the stack is invalid now
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

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
