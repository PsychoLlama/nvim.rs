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

use crate::{semsg_c, smsg_c};
use core::ffi::{c_char, c_int, c_long, c_uint};

use super::*;
use crate::highlight_group::HLF_E;
use crate::types::{FAIL, MAXPATHL, NUL, OK, OPT_LOCAL};

/// Try to recover `curbuf` from its swap file.
///
/// `checkext`: whether the buffer's own name may itself be a swap file name,
/// as it is for `nvim -r file.swp`.
pub unsafe fn ml_recover(checkext: bool) {
    unsafe {
        recoverymode.set(true);
        let called_from_main = (*curbuf.get()).b_ml.ml_mfp.is_null();

        let mut buf: *mut buf_T = core::ptr::null_mut();
        let mut mfp: *mut memfile_T = core::ptr::null_mut();
        let mut hp: *mut bhdr_T = core::ptr::null_mut();
        let mut fname_used: *mut c_char = core::ptr::null_mut();
        // Nothing was recovered yet, so a failure now leaves the buffer with
        // no memline at all.
        let mut serious_error = true;

        'theend: {
            let fname = if (*curbuf.get()).b_fname.is_null() {
                c"".as_ptr().cast_mut()
            } else {
                (*curbuf.get()).b_fname
            };
            // A name ending in ".s[a-w][a-z]" is taken to be the swap file
            // itself; otherwise its swap files are searched for.
            let directly = checkext && looks_like_swapfile(fname);
            fname_used = if directly {
                xstrdup(fname) // a copy for mf_open(), which consumes it
            } else {
                let Some(chosen) = choose_swapfile(fname) else {
                    break 'theend;
                };
                chosen
            };
            if fname_used.is_null() {
                break 'theend; // the user chose an invalid number
            }
            // When called from main() the storage structure still needs
            // initialising.
            if called_from_main && ml_open(curbuf.get()) == FAIL {
                getout(1);
            }

            // A buffer structure for the swap file being recovered. Only the
            // memline in it is really used.
            buf = xmalloc(size_of::<buf_T>()) as *mut buf_T;
            (*buf).b_ml.ml_stack_size = 0; // no stack yet
            (*buf).b_ml.ml_stack = core::ptr::null_mut();
            (*buf).b_ml.ml_stack_top = 0; // nothing in the stack
            (*buf).b_ml.ml_line_lnum = 0; // no cached line
            (*buf).b_ml.ml_line_offset = 0;
            (*buf).b_ml.ml_locked = core::ptr::null_mut(); // no locked block
            (*buf).b_ml.ml_flags = 0;

            // Open the memfile on the old swap file. `mf_open` consumes the
            // name, so keep a copy of it for the messages.
            let kept = xstrdup(fname_used);
            mfp = mf_open(fname_used, O_RDONLY);
            fname_used = kept;
            if mfp.is_null() || (*mfp).mf_fd < 0 {
                semsg_c!(gettext(c"E306: Cannot open %s".as_ptr()), fname_used);
                break 'theend;
            }
            (*buf).b_ml.ml_mfp = mfp;

            // The page size `mf_open` picked need not be the one the swap
            // file was written with; the real one is in block zero. Reading
            // block zero needs *a* page size, so use the smallest one a swap
            // file can have, and correct it below.
            (*mfp).mf_page_size = MIN_SWAP_PAGE_SIZE;

            let hl_id = HLF_E;
            msg_ext_set_kind(c"emsg".as_ptr());
            hp = mf_get(mfp, 0, 1);
            if hp.is_null() {
                msg_start();
                msg_puts_hl(
                    gettext(c"Unable to read block 0 from ".as_ptr()),
                    hl_id,
                    true,
                );
                msg_outtrans(mf_fname(mfp), hl_id, true);
                msg_puts_hl(
                    gettext(
                        c"\nMaybe no changes were made or Nvim did not update the swap file."
                            .as_ptr(),
                    ),
                    hl_id,
                    true,
                );
                msg_end();
                break 'theend;
            }
            let mut b0p = (*hp).bh_data as *mut ZeroBlock;
            if strncmp((*b0p).b0_version.as_ptr(), c"VIM 3.0".as_ptr(), 7) == 0 {
                msg_start();
                msg_outtrans(mf_fname(mfp), 0, true);
                msg_puts_hl(
                    gettext(c" cannot be used with this version of Nvim.\n".as_ptr()),
                    0,
                    true,
                );
                msg_puts_hl(gettext(c"Use Vim version 3.0.\n".as_ptr()), 0, true);
                msg_end();
                break 'theend;
            }
            if !ml_check_b0_id(&*b0p) {
                semsg_c!(
                    gettext(c"E307: %s does not look like a Nvim swap file".as_ptr()),
                    mf_fname(mfp),
                );
                break 'theend;
            }
            if b0_magic_wrong(&*b0p) {
                msg_start();
                msg_outtrans(mf_fname(mfp), hl_id, true);
                msg_puts_hl(
                    gettext(c" cannot be used on this computer.\n".as_ptr()),
                    hl_id,
                    true,
                );
                msg_puts_hl(gettext(c"The file was created on ".as_ptr()), hl_id, true);
                // Terminate the name field, so that printing the host name
                // cannot run off the end of a corrupted one.
                (*b0p).b0_fname[0] = NUL as c_char;
                msg_puts_hl((*b0p).b0_hname.as_ptr(), hl_id, true);
                msg_puts_hl(
                    gettext(c",\nor the file has been damaged.".as_ptr()),
                    hl_id,
                    true,
                );
                msg_end();
                break 'theend;
            }

            // The guessed page size was wrong, so the highest block number
            // in the file has to be worked out again.
            let recorded_page_size = b0_read_number(&(*b0p).b0_page_size) as c_uint;
            if (*mfp).mf_page_size != recorded_page_size {
                let previous_page_size = (*mfp).mf_page_size;
                mf_new_page_size(mfp, recorded_page_size);
                if (*mfp).mf_page_size < previous_page_size {
                    msg_start();
                    msg_outtrans(mf_fname(mfp), hl_id, true);
                    msg_puts_hl(
                        gettext(
                            c" has been damaged (page size is smaller than minimum value).\n"
                                .as_ptr(),
                        ),
                        hl_id,
                        true,
                    );
                    msg_end();
                    break 'theend;
                }
                let size = lseek((*mfp).mf_fd, 0, SEEK_END);
                // Zero means no file, or an empty one.
                (*mfp).mf_blocknr_max = if size <= 0 {
                    0
                } else {
                    size / (*mfp).mf_page_size as off_T
                } as blocknr_T;
                (*mfp).mf_infile_count = (*mfp).mf_blocknr_max;

                // Block zero's own buffer was allocated at the guessed size.
                let bigger = xmalloc((*mfp).mf_page_size as size_t);
                memmove(bigger, (*hp).bh_data, previous_page_size as size_t);
                xfree((*hp).bh_data);
                (*hp).bh_data = bigger;
                b0p = bigger as *mut ZeroBlock;
            }

            // Given the swap file's name directly, the buffer takes its name
            // from what the swap file says it belongs to.
            if directly {
                expand_env(
                    (*b0p).b0_fname.as_mut_ptr(),
                    NameBuff.ptr().cast(),
                    MAXPATHL,
                );
                if setfname(
                    curbuf.get(),
                    NameBuff.ptr().cast(),
                    core::ptr::null_mut(),
                    true,
                ) == FAIL
                {
                    break 'theend;
                }
            }

            msg_ext_set_kind(c"wmsg".as_ptr());
            msg_ext_skip_flush.set(true);
            home_replace(
                core::ptr::null(),
                mf_fname(mfp),
                NameBuff.ptr().cast(),
                MAXPATHL as size_t,
                true,
            );
            smsg_c!(
                0,
                gettext(c"Using swap file \"%s\"".as_ptr()),
                NameBuff.ptr(),
            );
            if !buf_spname(curbuf.get()).is_null() {
                xstrlcpy(
                    NameBuff.ptr().cast(),
                    buf_spname(curbuf.get()),
                    MAXPATHL as size_t,
                );
            } else {
                home_replace(
                    core::ptr::null(),
                    (*curbuf.get()).b_ffname,
                    NameBuff.ptr().cast(),
                    MAXPATHL as size_t,
                    true,
                );
            }
            msg_putchar('\n' as c_int);
            smsg_c!(
                0,
                gettext(c"Original file \"%s\"".as_ptr()),
                NameBuff.ptr().cast::<c_char>(),
            );
            msg_putchar('\n' as c_int);
            msg_ext_skip_flush.set(false);

            // Compare the dates of the swap file and the original.
            let mtime = b0_read_number(&(*b0p).b0_mtime) as c_int;
            let mut org_file_info: FileInfo = core::mem::zeroed();
            let mut swp_file_info: FileInfo = core::mem::zeroed();
            if !(*curbuf.get()).b_ffname.is_null()
                && os_fileinfo((*curbuf.get()).b_ffname, &raw mut org_file_info)
                && ((os_fileinfo(mf_fname(mfp), &raw mut swp_file_info)
                    && org_file_info.stat.st_mtim.tv_sec > swp_file_info.stat.st_mtim.tv_sec)
                    || org_file_info.stat.st_mtim.tv_sec != mtime as _)
            {
                emsg(gettext(
                    c"E308: Warning: Original file may have been changed".as_ptr(),
                ));
            }
            ui_flush();

            // Take 'fileformat' and 'fileencoding' from block zero. The
            // encoding sits at the very end of the name field, behind a NUL,
            // so it is found by scanning back from there.
            let b0_ff = (*b0p).flags() & B0_FF_MASK;
            let mut b0_fenc: *mut c_char = core::ptr::null_mut();
            if (*b0p).flags() & B0_HAS_FENC != 0 {
                let name = (*b0p).b0_fname.as_mut_ptr();
                let end = name.offset(B0_FNAME_SIZE_NOCRYPT as isize);
                let mut p = end;
                while p > name && *p.offset(-1) as c_int != NUL {
                    p = p.offset(-1);
                }
                b0_fenc = xstrnsave(p, end.offset_from(p) as size_t);
            }

            // Release block zero. `b0p` is still read further down, for the
            // "process STILL RUNNING" note: `mf_put` only drops the block's
            // reference, and nothing here can make the memfile hand its page
            // back out.
            mf_put(mfp, hp, false, false);
            hp = core::ptr::null_mut();

            // Recovery is going ahead, so the buffer's current contents go.
            while (*curbuf.get()).b_ml.ml_flags & ML_EMPTY == 0 {
                ml_delete(1);
            }

            // Read the original file, to pick up 'fileformat', 'fileencoding'
            // and friends. Errors are ignored, and the text itself is not
            // used — except as the "unchanged?" comparison below.
            let mut orig_file_status = NOTDONE;
            if !(*curbuf.get()).b_ffname.is_null() {
                orig_file_status = readfile(
                    (*curbuf.get()).b_ffname,
                    core::ptr::null_mut(),
                    0,
                    0,
                    MAXLNUM as linenr_T,
                    core::ptr::null_mut(),
                    READ_NEW as c_int,
                    false,
                );
            }

            // What the swap file recorded wins over what the file suggests.
            if b0_ff != 0 {
                set_fileformat(b0_ff - 1, OPT_LOCAL as c_int);
            }
            if !b0_fenc.is_null() {
                set_option_value_give_err(
                    kOptFileencoding,
                    OptVal {
                        type_0: kOptValTypeString,
                        data: OptValData {
                            string: cstr_as_string(b0_fenc),
                        },
                    },
                    OPT_LOCAL as c_int,
                );
                xfree(b0_fenc.cast());
            }
            unchanged(curbuf.get(), true, true);

            serious_error = false;
            let Ok((lnum, error)) = recover_lines(buf, mfp, &mut hp) else {
                break 'theend;
            };

            // Compare the recovered contents with the original file's.
            // Lines 1 to lnum are what was recovered, lines lnum + 1 to
            // ml_line_count are the file's, and line ml_line_count + 1 is the
            // empty buffer's dummy line.
            if orig_file_status != OK || (*curbuf.get()).b_ml.ml_line_count != lnum * 2 + 1 {
                // Recovering an empty file gives two lines of which the first
                // is empty; that is not a modification.
                if !((*curbuf.get()).b_ml.ml_line_count == 2 && *ml_get(1) as c_int == NUL) {
                    changed_internal(curbuf.get());
                    buf_inc_changedtick(curbuf.get());
                }
            } else {
                for idx in 1..=lnum {
                    // One of the two lines has to be copied: fetching the
                    // other may flush it.
                    let p = xstrnsave(ml_get(idx), ml_get_len(idx) as size_t);
                    let same = strcmp(p, ml_get(idx + lnum)) == 0;
                    xfree(p.cast());
                    if !same {
                        changed_internal(curbuf.get());
                        buf_inc_changedtick(curbuf.get());
                        break;
                    }
                }
            }

            // Drop the original file's lines and the empty buffer's dummy
            // line; they are now past the end of what was recovered.
            while (*curbuf.get()).b_ml.ml_line_count > lnum
                && (*curbuf.get()).b_ml.ml_flags & ML_EMPTY == 0
            {
                ml_delete((*curbuf.get()).b_ml.ml_line_count);
            }
            (*curbuf.get()).b_flags |= BF_RECOVERED;
            check_cursor(curwin.get());

            msg_ext_skip_flush.set(!got_int.get());
            recoverymode.set(false);
            report_recovery(error, b0p, fname_used);
            redraw_curbuf_later(UPD_NOT_VALID);
        }

        msg_ext_skip_flush.set(false);
        xfree(fname_used.cast());
        recoverymode.set(false);
        if !mfp.is_null() {
            if !hp.is_null() {
                mf_put(mfp, hp, false, false);
            }
            mf_close(mfp, false); // also frees the swap file's name
        }
        if !buf.is_null() {
            // May be null: the swap file was never found.
            xfree((*buf).b_ml.ml_stack.cast());
            xfree(buf.cast());
        }
        if serious_error && called_from_main {
            ml_close(curbuf.get(), true_0);
        } else {
            apply_autocmds(
                EVENT_BUFREADPOST,
                core::ptr::null_mut(),
                (*curbuf.get()).b_fname,
                false,
                curbuf.get(),
            );
            apply_autocmds(
                EVENT_BUFWINENTER,
                core::ptr::null_mut(),
                (*curbuf.get()).b_fname,
                false,
                curbuf.get(),
            );
        }
    }
}

/// Whether this name is itself a swap file name: it ends in `.s`, a letter
/// from `a` to `w`, and any letter — the extensions `findswapname` permutes
/// through.
unsafe fn looks_like_swapfile(fname: *mut c_char) -> bool {
    unsafe {
        let len = strlen(fname) as isize;
        len >= 4
            && strncasecmp(fname.offset(len - 4), c".s".as_ptr(), 2) == 0
            && !vim_strchr(
                c"abcdefghijklmnopqrstuvw".as_ptr(),
                (*fname.offset(len - 2) as u8).to_ascii_lowercase() as c_int,
            )
            .is_null()
            && (*fname.offset(len - 1) as u8).is_ascii_alphabetic()
    }
}

/// Pick which of `fname`'s swap files to recover from: the only one there is,
/// or the one the user names out of a listing.
///
/// Returns the allocated name, or `None` to give up.
unsafe fn choose_swapfile(fname: *mut c_char) -> Option<*mut c_char> {
    unsafe {
        let count = recover_names(
            fname,
            false,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
        );
        if count == 0 {
            semsg_c!(gettext(c"E305: No swap file found for %s".as_ptr()), fname);
            return None;
        }
        let nr = if count == 1 {
            1
        } else {
            recover_names(fname, true, core::ptr::null_mut(), 0, core::ptr::null_mut());
            if !ui_has(kUIMessages) {
                msg_putchar('\n' as c_int);
            }
            let nr = prompt_for_input(
                gettext(c"Enter number of swap file to use (0 to quit): ".as_ptr()),
                0,
                false,
                core::ptr::null_mut(),
            );
            if nr < 1 || nr > count {
                return None;
            }
            nr
        };
        let mut fname_used: *mut c_char = core::ptr::null_mut();
        recover_names(fname, false, core::ptr::null_mut(), nr, &raw mut fname_used);
        Some(fname_used)
    }
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
    unsafe {
        let mut bnum: blocknr_T = 1; // start with block 1
        let mut page_count: c_uint = 1; // which is one page
        let mut lnum: linenr_T = 0; // append after line 0 in curbuf
        let mut line_count: linenr_T = 0;
        let mut idx = 0; // start with the first index in block 1
        let mut error = 0;
        (*buf).b_ml.ml_stack_top = 0;
        (*buf).b_ml.ml_stack = core::ptr::null_mut();
        (*buf).b_ml.ml_stack_size = 0;

        // Without a file to fall back on, a data block whose number went
        // negative (never written to the swap file) is simply lost.
        let mut cannot_open = (*curbuf.get()).b_ffname.is_null();

        let append = |lnum: &mut linenr_T, text: *const c_char| {
            ml_append(*lnum, text.cast_mut(), 0, true);
            *lnum += 1;
        };

        'walk: while !got_int.get() {
            'step: {
                if !hp.is_null() {
                    mf_put(mfp, *hp, false, false); // release the previous block
                }
                *hp = mf_get(mfp, bnum, page_count);
                if hp.is_null() {
                    if bnum == 1 {
                        semsg_c!(
                            gettext(c"E309: Unable to read block 1 from %s".as_ptr()),
                            mf_fname(mfp),
                        );
                        return Err(());
                    }
                    error += 1;
                    append(&mut lnum, gettext(c"???MANY LINES MISSING".as_ptr()));
                } else if (*((**hp).bh_data as *mut PointerBlock)).pb_id == PTR_ID as uint16_t {
                    let pp = (**hp).bh_data as *mut PointerBlock;
                    // The counts in the header have to fit the page size this
                    // build uses, or the entries cannot be walked at all.
                    let count_max = PointerBlock::count_max((*mfp).mf_page_size);
                    let mut ptr_block_error = false;
                    if (*pp).pb_count_max != count_max {
                        ptr_block_error = true;
                        (*pp).pb_count_max = count_max;
                    }
                    if (*pp).pb_count > (*pp).pb_count_max {
                        ptr_block_error = true;
                        (*pp).pb_count = (*pp).pb_count_max;
                    }
                    if ptr_block_error {
                        emsg(gettext(c"E1364: Warning: Pointer block corrupted".as_ptr()));
                    }

                    // The first time down this block, its entries should
                    // account for exactly the line count promised above it.
                    if idx == 0 && line_count != 0 {
                        for i in 0..(*pp).pb_count as usize {
                            line_count -= (&raw mut (*pp).pb_pointer)
                                .cast::<PointerEntry>()
                                .add(i)
                                .read()
                                .pe_line_count;
                        }
                        if line_count != 0 {
                            error += 1;
                            append(&mut lnum, gettext(c"???LINE COUNT WRONG".as_ptr()));
                        }
                    }

                    if (*pp).pb_count == 0 {
                        append(&mut lnum, gettext(c"???EMPTY BLOCK".as_ptr()));
                        error += 1;
                    } else if idx < (*pp).pb_count as c_int {
                        let pe = (&raw mut (*pp).pb_pointer)
                            .cast::<PointerEntry>()
                            .add(idx as usize)
                            .read();
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
                                    || readfile(
                                        (*curbuf.get()).b_ffname,
                                        core::ptr::null_mut(),
                                        lnum,
                                        pe.pe_old_lnum - 1,
                                        line_count,
                                        core::ptr::null_mut(),
                                        0,
                                        false,
                                    ) != OK
                                {
                                    cannot_open = true;
                                } else {
                                    lnum += line_count;
                                }
                            }
                            if cannot_open {
                                error += 1;
                                append(&mut lnum, gettext(c"???LINES MISSING".as_ptr()));
                            }
                            idx += 1; // same block again, for the next index
                            break 'step;
                        }

                        // One block deeper in the tree.
                        let top = ml_add_stack(buf);
                        let ip = (*buf).b_ml.ml_stack.offset(top as isize);
                        (*ip).ip_bnum = bnum;
                        (*ip).ip_index = idx;

                        bnum = pe.pe_bnum;
                        line_count = pe.pe_line_count;
                        page_count = pe.pe_page_count as c_uint;
                        // `pe_page_count` sizes the allocation `mf_get` makes,
                        // so a bogus value (0x40000000, say) would ask for
                        // gigabytes. It must be at least one page, and the
                        // block must lie inside the file.
                        if page_count < 1
                            || bnum + page_count as blocknr_T > (*mfp).mf_blocknr_max + 1
                        {
                            error += 1;
                            append(&mut lnum, gettext(c"???ILLEGAL BLOCK NUMBER".as_ptr()));
                            // Skip this entry and pop back up, to recover
                            // whatever else there is.
                            idx = (*ip).ip_index + 1;
                            bnum = (*ip).ip_bnum;
                            page_count = 1;
                            (*buf).b_ml.ml_stack_top -= 1;
                            break 'step;
                        }
                        idx = 0;
                        break 'step;
                    }
                } else {
                    let dp = (**hp).bh_data as *mut DataBlock;
                    if (*dp).db_id != DATA_ID as uint16_t {
                        if bnum == 1 {
                            semsg_c!(
                                gettext(c"E310: Block 1 ID wrong (%s not a .swp file?)".as_ptr()),
                                mf_fname(mfp),
                            );
                            return Err(());
                        }
                        error += 1;
                        append(&mut lnum, gettext(c"???BLOCK MISSING".as_ptr()));
                    } else {
                        // A data block: append every line in it.
                        let mut has_error = false;

                        // The block's length has to match the page count the
                        // pointer block promised; if it does not, trust the
                        // pointer block.
                        if page_count.wrapping_mul((*mfp).mf_page_size) != (*dp).db_txt_end {
                            append(
                                &mut lnum,
                                gettext(
                                    c"??? from here until ???END lines may be messed up".as_ptr(),
                                ),
                            );
                            error += 1;
                            has_error = true;
                            (*dp).db_txt_end = page_count.wrapping_mul((*mfp).mf_page_size);
                        }

                        // Make sure the block ends in a NUL, so that copying
                        // the last line out of it cannot run past the end.
                        *(dp as *mut c_char).offset((*dp).db_txt_end as isize - 1) = NUL as c_char;

                        // Likewise for the line count.
                        if line_count as c_long != (*dp).db_line_count {
                            append(
                                &mut lnum,
                                gettext(
                                    c"??? from here until ???END lines may have been inserted/deleted"
                                        .as_ptr(),
                                ),
                            );
                            error += 1;
                            has_error = true;
                        }

                        let mut did_questions = false;
                        for i in 0..(*dp).db_line_count {
                            let index = (&raw mut (*dp).db_index)
                                .cast::<c_uint>()
                                .offset(i as isize);
                            if index as *mut c_char
                                >= (dp as *mut c_char).offset((*dp).db_txt_start as isize)
                            {
                                // The line count must be wrong: the index
                                // array has run into the text.
                                error += 1;
                                append(&mut lnum, gettext(c"??? lines may be missing".as_ptr()));
                                break;
                            }
                            let txt_start = (index.read() & DB_INDEX_MASK) as c_int;
                            let text = if txt_start <= HEADER_SIZE as c_int
                                || txt_start >= (*dp).db_txt_end as c_int
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
                                (dp as *mut c_char).offset(txt_start as isize)
                            };
                            append(&mut lnum, text);
                        }
                        if has_error {
                            append(&mut lnum, gettext(c"???END".as_ptr()));
                        }
                    }
                }

                if (*buf).b_ml.ml_stack_top == 0 {
                    break 'walk; // finished
                }

                // One block back up the tree, and on to the next index.
                (*buf).b_ml.ml_stack_top -= 1;
                let ip = (*buf)
                    .b_ml
                    .ml_stack
                    .offset((*buf).b_ml.ml_stack_top as isize);
                bnum = (*ip).ip_bnum;
                idx = (*ip).ip_index + 1;
                page_count = 1;
            }
            line_breakcheck();
        }

        Ok((lnum, error))
    }
}

/// Say how it went, and what the user should do next.
unsafe fn report_recovery(error: c_int, b0p: *const ZeroBlock, fname_used: *const c_char) {
    unsafe {
        if got_int.get() {
            emsg(gettext(c"E311: Recovery Interrupted".as_ptr()));
            return;
        }
        if error != 0 {
            *no_wait_return.ptr() += 1;
            msg_ext_set_kind(c"emsg".as_ptr());
            msg(c">>>>>>>>>>>>>\n".as_ptr(), 0);
            emsg(gettext(
                c"E312: Errors detected while recovering; look for lines starting with ???"
                    .as_ptr(),
            ));
            *no_wait_return.ptr() -= 1;
            msg_putchar('\n' as c_int);
            msg(
                gettext(c"See \":help E312\" for more information.".as_ptr()),
                0,
            );
            msg(c"\n>>>>>>>>>>>>>".as_ptr(), 0);
            return;
        }

        msg_ext_set_kind(c"wmsg".as_ptr());
        if (*curbuf.get()).b_changed != 0 {
            msg(
                gettext(c"Recovery completed. You should check if everything is OK.".as_ptr()),
                0,
            );
            msg_puts(gettext(
                c"\n(You might want to write out this file under another name\n".as_ptr(),
            ));
            msg_puts(gettext(
                c"and run diff with the original file to check for changes)".as_ptr(),
            ));
        } else {
            msg(
                gettext(c"Recovery completed. Buffer contents equals file contents.".as_ptr()),
                0,
            );
        }
        msg_puts(gettext(
            c"\nYou may want to delete the .swp file now.".as_ptr(),
        ));
        if swapfile_proc_running(&*b0p, fname_used) != 0 {
            // There may be a live Nvim on the same file; the user may want to
            // go and kill it.
            msg_puts(gettext(c"\nNote: process STILL RUNNING: ".as_ptr()));
            msg_outnum((*b0p).pid() as c_int);
        }
        if !ui_has(kUIMessages) {
            msg_puts(c"\n\n".as_ptr());
        }
        cmdline_row.set(msg_row.get());
    }
}

/// Sync every buffer's memline.
///
/// `check_file`: also check that the original file still exists and is
/// unchanged. `check_char`: stop as soon as a character is typed, having
/// synced at least one block.
pub unsafe fn ml_sync_all(check_file: c_int, check_char: c_int, do_fsync: bool) {
    unsafe {
        let mut buf = firstbuf.get();
        while !buf.is_null() {
            if !(*buf).b_ml.ml_mfp.is_null() && !mf_fname((*buf).b_ml.ml_mfp).is_null() {
                ml_flush_line(buf, false); // flush the buffered line
                ml_find_line(buf, 0, ML_FLUSH as c_int); // flush the locked block

                if bufIsChanged(buf)
                    && check_file != 0
                    && mf_need_trans((*buf).b_ml.ml_mfp)
                    && !(*buf).b_ffname.is_null()
                {
                    // If the original file is gone or has changed, preserve
                    // now, to get rid of all the negative numbered blocks.
                    let mut file_info: FileInfo = core::mem::zeroed();
                    if !os_fileinfo((*buf).b_ffname, &raw mut file_info)
                        || file_info.stat.st_mtim.tv_sec != (*buf).b_mtime_read
                        || file_info.stat.st_mtim.tv_nsec != (*buf).b_mtime_read_ns
                        || os_fileinfo_size(&raw mut file_info) != (*buf).b_orig_size
                    {
                        ml_preserve(buf, false, do_fsync);
                        did_check_timestamps.set(false);
                        need_check_timestamps.set(true); // give the message later
                    }
                }

                if (*(*buf).b_ml.ml_mfp).mf_dirty == MfDirty::Yes {
                    mf_sync(
                        (*buf).b_ml.ml_mfp,
                        (if check_char != 0 {
                            MFS_STOP as c_int
                        } else {
                            0
                        }) | (if do_fsync && bufIsChanged(buf) {
                            MFS_FLUSH as c_int
                        } else {
                            0
                        }),
                    );
                    if check_char != 0 && os_char_avail() {
                        break; // a character is available now
                    }
                }
            }
            buf = (*buf).b_next;
        }
    }
}

/// Sync one buffer, including its negative numbered blocks, so that
/// afterwards everything is in the swap file.
///
/// This is `:preserve`, and what happens when the original file has been
/// changed or deleted. `message` reports whether it worked.
pub unsafe fn ml_preserve(buf: *mut buf_T, message: bool, do_fsync: bool) {
    unsafe {
        let mfp = (*buf).b_ml.ml_mfp;
        if mfp.is_null() || mf_fname(mfp).is_null() {
            if message {
                emsg(gettext(
                    c"E313: Cannot preserve, there is no swap file".as_ptr(),
                ));
            }
            return;
        }

        // Only an interrupt from here on counts, not one from before.
        let got_int_save = got_int.get();
        got_int.set(false);

        ml_flush_line(buf, false); // flush the buffered line
        ml_find_line(buf, 0, ML_FLUSH as c_int); // flush the locked block
        let mut status = mf_sync(
            mfp,
            MFS_ALL as c_int | if do_fsync { MFS_FLUSH as c_int } else { 0 },
        );
        (*buf).b_ml.ml_stack_top = 0; // the stack is invalid after MFS_ALL

        // Some data blocks may have gone from a negative to a positive block
        // number, which means the pointer blocks referring to them need
        // updating. Which pointer blocks those are is not recorded, so every
        // data block is visited until no translations are left (or the end of
        // the file is reached, which can only happen when a write failed —
        // a full file system, say). `ml_find_line` does the work, translating
        // the negative numbers as it fetches each block's first line.
        'theend: {
            if mf_need_trans(mfp) && !got_int.get() {
                let mut lnum: linenr_T = 1;
                while mf_need_trans(mfp) && lnum <= (*buf).b_ml.ml_line_count {
                    if ml_find_line(buf, lnum, ML_FIND as c_int).is_null() {
                        status = FAIL;
                        break 'theend;
                    }
                    lnum = (*buf).b_ml.ml_locked_high + 1;
                }
                ml_find_line(buf, 0, ML_FLUSH as c_int); // flush the locked block
                // Sync the pointer blocks that were just updated.
                if mf_sync(
                    mfp,
                    MFS_ALL as c_int | if do_fsync { MFS_FLUSH as c_int } else { 0 },
                ) == FAIL
                {
                    status = FAIL;
                }
                (*buf).b_ml.ml_stack_top = 0; // the stack is invalid now
            }
        }
        got_int.set(got_int.get() | got_int_save);

        if message {
            if status == OK {
                msg(gettext(c"File preserved".as_ptr()), 0);
            } else {
                emsg(gettext(c"E314: Preserve failed".as_ptr()));
            }
        }
    }
}
