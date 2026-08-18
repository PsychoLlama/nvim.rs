//! `u_read_undo`: reading an undo tree back in, and grafting it onto a
//! buffer whose contents still hash the same.

use super::file::*;
use super::format::*;
use super::tree::*;
use super::*;
use crate::{semsg_c, smsg_c};

pub unsafe fn u_read_undo(
    mut name: *mut c_char,
    mut hash: *const uint8_t,
    mut orig_name: *const c_char,
) {
    let mut bi: bufinfo_T = bufinfo_T {
        bi_buf: ptr::null_mut(),
        bi_fp: ptr::null_mut(),
    };
    let mut magic_buf: [c_char; 9] = [0; 9];
    let mut version: c_int = 0;
    let mut read_hash: [uint8_t; 32] = [0; 32];
    let mut line_count: linenr_T = 0;
    let mut str_len: c_int = 0;
    let mut line_lnum: linenr_T = 0;
    let mut line_colnr: colnr_T = 0;
    let mut old_header_seq: c_int = 0;
    let mut new_header_seq: c_int = 0;
    let mut cur_header_seq: c_int = 0;
    let mut num_head: c_int = 0;
    let mut seq_last: c_int = 0;
    let mut seq_cur: c_int = 0;
    let mut seq_time: time_t = 0;
    let mut last_save_nr: c_int = 0;
    let mut num_read_uhps: c_int = 0;
    let mut c: c_int = 0;
    let mut old_idx: int16_t = 0;
    let mut new_idx: int16_t = 0;
    let mut cur_idx: int16_t = 0;
    let mut uhp_table: *mut *mut u_header_T = ptr::null_mut();
    let mut line_ptr: *mut c_char = ptr::null_mut();
    let mut file_name: *mut c_char = ptr::null_mut();
    if name.is_null() {
        file_name = u_get_undo_file_name((*curbuf.get()).b_ffname, true);
        if file_name.is_null() {
            return;
        }
        let mut file_info_orig: FileInfo = FileInfo {
            stat: uv_stat_t {
                st_dev: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_ino: 0,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_flags: 0,
                st_gen: 0,
                st_atim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_birthtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
        };
        let mut file_info_undo: FileInfo = FileInfo {
            stat: uv_stat_t {
                st_dev: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_ino: 0,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_flags: 0,
                st_gen: 0,
                st_atim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_birthtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
        };
        if os_fileinfo(orig_name, &raw mut file_info_orig)
            && os_fileinfo(file_name, &raw mut file_info_undo)
            && file_info_orig.stat.st_uid != file_info_undo.stat.st_uid
            && file_info_undo.stat.st_uid != getuid() as uint64_t
        {
            if p_verbose.get() > 0 {
                verbose_enter();
                smsg_c!(
                    0,
                    gettext(c"Not reading undo file, owner differs: %s".as_ptr()),
                    file_name,
                );
                verbose_leave();
            }
            return;
        }
    } else {
        file_name = name;
    }
    if p_verbose.get() > 0 {
        verbose_enter();
        smsg_c!(0, gettext(c"Reading undo file: %s".as_ptr()), file_name);
        verbose_leave();
    }
    let mut fp: *mut FILE = os_fopen(file_name, c"r".as_ptr());
    '_theend: {
        '_error: {
            if fp.is_null() {
                if !name.is_null() || p_verbose.get() > 0 {
                    semsg_c!(
                        gettext(c"E822: Cannot open undo file for reading: %s".as_ptr()),
                        file_name,
                    );
                }
            } else {
                bi = bufinfo_T {
                    bi_buf: curbuf.get(),
                    bi_fp: fp,
                };
                magic_buf = [0; 9];
                if fread(
                    &raw mut magic_buf as *mut c_char as *mut c_void,
                    UF_START_MAGIC_LEN as size_t,
                    1,
                    fp,
                ) != 1
                    || memcmp(
                        &raw mut magic_buf as *mut c_char as *const c_void,
                        UF_START_MAGIC.as_ptr() as *const c_void,
                        UF_START_MAGIC_LEN as size_t,
                    ) != 0
                {
                    semsg_c!(gettext(c"E823: Not an undo file: %s".as_ptr()), file_name);
                } else {
                    version = get2c(fp);
                    if version != UF_VERSION {
                        semsg_c!(
                            gettext(c"E824: Incompatible undo file: %s".as_ptr()),
                            file_name,
                        );
                    } else {
                        read_hash = [0; 32];
                        if !undo_read(
                            &raw mut bi,
                            &raw mut read_hash as *mut uint8_t,
                            UNDO_HASH_SIZE as c_int as size_t,
                        ) {
                            corruption_error(c"hash".as_ptr(), file_name);
                        } else {
                            line_count = undo_read_4c(&raw mut bi) as linenr_T;
                            if memcmp(
                                hash as *const c_void,
                                &raw mut read_hash as *mut uint8_t as *const c_void,
                                UNDO_HASH_SIZE as c_int as size_t,
                            ) != 0
                                || line_count != (*curbuf.get()).b_ml.ml_line_count
                            {
                                if p_verbose.get() > 0 || !name.is_null() {
                                    if name.is_null() {
                                        verbose_enter();
                                    }
                                    give_warning(
                                        gettext(
                                            c"File contents changed, cannot use undo info".as_ptr(),
                                        ),
                                        true,
                                        true,
                                    );
                                    if name.is_null() {
                                        verbose_leave();
                                    }
                                }
                            } else {
                                str_len = undo_read_4c(&raw mut bi);
                                if str_len >= 0 {
                                    if str_len > 0 {
                                        line_ptr = undo_read_string(&raw mut bi, str_len as size_t);
                                    }
                                    line_lnum = undo_read_4c(&raw mut bi) as linenr_T;
                                    line_colnr = undo_read_4c(&raw mut bi);
                                    if line_lnum < 0 || line_colnr < 0 {
                                        corruption_error(c"line lnum/col".as_ptr(), file_name);
                                    } else {
                                        old_header_seq = undo_read_4c(&raw mut bi);
                                        new_header_seq = undo_read_4c(&raw mut bi);
                                        cur_header_seq = undo_read_4c(&raw mut bi);
                                        num_head = undo_read_4c(&raw mut bi);
                                        seq_last = undo_read_4c(&raw mut bi);
                                        seq_cur = undo_read_4c(&raw mut bi);
                                        seq_time = undo_read_time(&raw mut bi);
                                        last_save_nr = 0;
                                        loop {
                                            let mut len: c_int = undo_read_byte(&raw mut bi);
                                            if len == 0 || len == EOF {
                                                break;
                                            }
                                            let mut what: c_int = undo_read_byte(&raw mut bi);
                                            match what {
                                                UF_LAST_SAVE_NR => {
                                                    last_save_nr = undo_read_4c(&raw mut bi);
                                                }
                                                _ => loop {
                                                    len -= 1;
                                                    if len < 0 {
                                                        break;
                                                    }
                                                    undo_read_byte(&raw mut bi);
                                                },
                                            }
                                        }
                                        if num_head > 0 {
                                            if (num_head as size_t)
                                                < (SIZE_MAX as usize)
                                                    .wrapping_div(size_of::<*mut u_header_T>())
                                            {
                                                uhp_table = xmalloc(
                                                    (num_head as size_t)
                                                        .wrapping_mul(size_of::<*mut u_header_T>()),
                                                )
                                                    as *mut *mut u_header_T;
                                            }
                                        }
                                        num_read_uhps = 0;
                                        c = 0;
                                        loop {
                                            c = undo_read_2c(&raw mut bi);
                                            if c != UF_HEADER_MAGIC {
                                                break;
                                            }
                                            if num_read_uhps >= num_head {
                                                corruption_error(
                                                    c"num_head too small".as_ptr(),
                                                    file_name,
                                                );
                                                break '_error;
                                            } else {
                                                let mut uhp: *mut u_header_T =
                                                    unserialize_uhp(&raw mut bi, file_name);
                                                if uhp.is_null() {
                                                    break '_error;
                                                }
                                                let c2rust_fresh2 = num_read_uhps;
                                                num_read_uhps = num_read_uhps + 1;
                                                let c2rust_lvalue_ptr = &raw mut *uhp_table
                                                    .offset(c2rust_fresh2 as isize);
                                                *c2rust_lvalue_ptr = uhp;
                                            }
                                        }
                                        if num_read_uhps != num_head {
                                            corruption_error(c"num_head".as_ptr(), file_name);
                                        } else if c != UF_HEADER_END_MAGIC {
                                            corruption_error(c"end marker".as_ptr(), file_name);
                                        } else {
                                            old_idx = -1 as int16_t;
                                            new_idx = -1 as int16_t;
                                            cur_idx = -1 as int16_t;
                                            let mut i: c_int = 0;
                                            while i < num_head {
                                                let mut uhp_0: *mut u_header_T =
                                                    *uhp_table.offset(i as isize);
                                                if !uhp_0.is_null() {
                                                    let mut j: c_int = 0;
                                                    while j < num_head {
                                                        if !(*uhp_table.offset(j as isize))
                                                            .is_null()
                                                            && i != j
                                                            && (**uhp_table.offset(i as isize))
                                                                .uh_seq
                                                                == (**uhp_table.offset(j as isize))
                                                                    .uh_seq
                                                        {
                                                            corruption_error(
                                                                c"duplicate uh_seq".as_ptr(),
                                                                file_name,
                                                            );
                                                            break '_error;
                                                        } else {
                                                            j += 1;
                                                        }
                                                    }
                                                    let seq: c_int = (*uhp_0).uh_next.seq;
                                                    (*uhp_0).uh_next.ptr = ptr::null_mut();
                                                    let mut j_0: c_int = 0;
                                                    while j_0 < num_head {
                                                        if !(*uhp_table.offset(j_0 as isize))
                                                            .is_null()
                                                            && i != j_0
                                                            && (**uhp_table.offset(j_0 as isize))
                                                                .uh_seq
                                                                == seq
                                                        {
                                                            (*uhp_0).uh_next.ptr =
                                                                *uhp_table.offset(j_0 as isize);
                                                            break;
                                                        } else {
                                                            j_0 += 1;
                                                        }
                                                    }
                                                    let seq_0: c_int = (*uhp_0).uh_prev.seq;
                                                    (*uhp_0).uh_prev.ptr = ptr::null_mut();
                                                    let mut j_1: c_int = 0;
                                                    while j_1 < num_head {
                                                        if !(*uhp_table.offset(j_1 as isize))
                                                            .is_null()
                                                            && i != j_1
                                                            && (**uhp_table.offset(j_1 as isize))
                                                                .uh_seq
                                                                == seq_0
                                                        {
                                                            (*uhp_0).uh_prev.ptr =
                                                                *uhp_table.offset(j_1 as isize);
                                                            break;
                                                        } else {
                                                            j_1 += 1;
                                                        }
                                                    }
                                                    let seq_1: c_int = (*uhp_0).uh_alt_next.seq;
                                                    (*uhp_0).uh_alt_next.ptr = ptr::null_mut();
                                                    let mut j_2: c_int = 0;
                                                    while j_2 < num_head {
                                                        if !(*uhp_table.offset(j_2 as isize))
                                                            .is_null()
                                                            && i != j_2
                                                            && (**uhp_table.offset(j_2 as isize))
                                                                .uh_seq
                                                                == seq_1
                                                        {
                                                            (*uhp_0).uh_alt_next.ptr =
                                                                *uhp_table.offset(j_2 as isize);
                                                            break;
                                                        } else {
                                                            j_2 += 1;
                                                        }
                                                    }
                                                    let seq_2: c_int = (*uhp_0).uh_alt_prev.seq;
                                                    (*uhp_0).uh_alt_prev.ptr = ptr::null_mut();
                                                    let mut j_3: c_int = 0;
                                                    while j_3 < num_head {
                                                        if !(*uhp_table.offset(j_3 as isize))
                                                            .is_null()
                                                            && i != j_3
                                                            && (**uhp_table.offset(j_3 as isize))
                                                                .uh_seq
                                                                == seq_2
                                                        {
                                                            (*uhp_0).uh_alt_prev.ptr =
                                                                *uhp_table.offset(j_3 as isize);
                                                            break;
                                                        } else {
                                                            j_3 += 1;
                                                        }
                                                    }
                                                    if old_header_seq > 0
                                                        && (old_idx as c_int) < 0
                                                        && (*uhp_0).uh_seq == old_header_seq
                                                    {
                                                        debug_assert!(i <= 32767, "i <= INT16_MAX");
                                                        old_idx = i as int16_t;
                                                    }
                                                    if new_header_seq > 0
                                                        && (new_idx as c_int) < 0
                                                        && (*uhp_0).uh_seq == new_header_seq
                                                    {
                                                        debug_assert!(i <= 32767, "i <= INT16_MAX");
                                                        new_idx = i as int16_t;
                                                    }
                                                    if cur_header_seq > 0
                                                        && (cur_idx as c_int) < 0
                                                        && (*uhp_0).uh_seq == cur_header_seq
                                                    {
                                                        debug_assert!(i <= 32767, "i <= INT16_MAX");
                                                        cur_idx = i as int16_t;
                                                    }
                                                }
                                                i += 1;
                                            }
                                            u_blockfree(curbuf.get());
                                            (*curbuf.get()).b_u_oldhead = if (old_idx as c_int) < 0
                                            {
                                                ptr::null_mut()
                                            } else {
                                                *uhp_table.offset(old_idx as isize)
                                            };
                                            (*curbuf.get()).b_u_newhead = if (new_idx as c_int) < 0
                                            {
                                                ptr::null_mut()
                                            } else {
                                                *uhp_table.offset(new_idx as isize)
                                            };
                                            (*curbuf.get()).b_u_curhead = if (cur_idx as c_int) < 0
                                            {
                                                ptr::null_mut()
                                            } else {
                                                *uhp_table.offset(cur_idx as isize)
                                            };
                                            (*curbuf.get()).b_u_line_ptr = line_ptr;
                                            (*curbuf.get()).b_u_line_lnum = line_lnum;
                                            (*curbuf.get()).b_u_line_colnr = line_colnr;
                                            (*curbuf.get()).b_u_numhead = num_head;
                                            (*curbuf.get()).b_u_seq_last = seq_last;
                                            (*curbuf.get()).b_u_seq_cur = seq_cur;
                                            (*curbuf.get()).b_u_time_cur = seq_time;
                                            (*curbuf.get()).b_u_save_nr_last = last_save_nr;
                                            (*curbuf.get()).b_u_save_nr_cur = last_save_nr;
                                            (*curbuf.get()).b_u_synced = true;
                                            xfree(uhp_table as *mut c_void);
                                            if !name.is_null() {
                                                smsg_c!(
                                                    0,
                                                    gettext(
                                                        c"Finished reading undo file %s".as_ptr(),
                                                    ),
                                                    file_name,
                                                );
                                            }
                                            break '_theend;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        xfree(line_ptr as *mut c_void);
        if !uhp_table.is_null() {
            let mut i_0: c_int = 0;
            while i_0 < num_read_uhps {
                if !(*uhp_table.offset(i_0 as isize)).is_null() {
                    u_free_uhp(*uhp_table.offset(i_0 as isize));
                }
                i_0 += 1;
            }
            xfree(uhp_table as *mut c_void);
        }
    }
    if !fp.is_null() {
        fclose(fp);
    }
    if file_name != name {
        xfree(file_name as *mut c_void);
    }
}
