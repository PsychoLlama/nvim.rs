//! The `.un~` file: where it lives, what it hashes, and how its
//! records are laid down and picked back up.

use super::format::*;
use super::tree::*;
use super::*;
use crate::semsg_c;

pub unsafe fn u_compute_hash(mut buf: *mut buf_T, mut hash: *mut uint8_t) {
    let mut ctx = Sha256::new();
    let mut lnum: linenr_T = 1;
    while lnum <= (*buf).b_ml.ml_line_count {
        let mut p: *mut c_char = ml_get_buf(buf, lnum);
        // Include the terminating NUL as a line separator.
        ctx.update(::core::slice::from_raw_parts(p as *const u8, strlen(p) + 1));
        lnum += 1;
    }
    ::core::slice::from_raw_parts_mut(hash, SHA256_SUM_SIZE).copy_from_slice(&ctx.finish());
}
pub unsafe fn u_get_undo_file_name(buf_ffname: *const c_char, reading: bool) -> *mut c_char {
    let mut ffname: *const c_char = buf_ffname;
    if ffname.is_null() {
        return ptr::null_mut();
    }
    let mut fname_buf: [c_char; 4096] = [0; 4096];
    if resolve_symlink(ffname, &raw mut fname_buf as *mut c_char) == OK {
        ffname = &raw mut fname_buf as *mut c_char;
    }
    let mut dir_name: [c_char; 4097] = [0; 4097];
    let mut munged_name: *mut c_char = ptr::null_mut();
    let mut undo_file_name: *mut c_char = ptr::null_mut();
    let mut dirp: *mut c_char = p_udir.get();
    while *dirp as c_int != NUL {
        let mut dir_len: size_t = copy_option_part(
            &raw mut dirp,
            &raw mut dir_name as *mut c_char,
            MAXPATHL as size_t,
            c",".as_ptr() as *mut c_char,
        );
        if dir_len == 1 && dir_name[0] as c_int == '.' as c_int {
            let ffname_len: size_t = strlen(ffname);
            undo_file_name = xmalloc(ffname_len.wrapping_add(6)) as *mut c_char;
            memmove(
                undo_file_name as *mut c_void,
                ffname as *const c_void,
                ffname_len.wrapping_add(1),
            );
            let tail: *mut c_char = path_tail(undo_file_name);
            let tail_len: size_t = strlen(tail);
            memmove(
                tail.offset(1) as *mut c_void,
                tail as *const c_void,
                tail_len.wrapping_add(1),
            );
            *tail = '.' as c_char;
            memmove(
                tail.add(tail_len).offset(1) as *mut c_void,
                c".un~".as_ptr() as *const c_void,
                size_of::<[c_char; 5]>(),
            );
        } else {
            dir_name[dir_len as usize] = NUL as c_char;
            let mut p: *mut c_char =
                (&raw mut dir_name as *mut c_char).add(dir_len.wrapping_sub(1));
            while dir_len > 1 && vim_ispathsep(*p as c_int) {
                let c2rust_fresh1 = p;
                p = p.offset(-1);
                *c2rust_fresh1 = NUL as c_char;
            }
            let mut has_directory: bool = os_isdir(&raw mut dir_name as *mut c_char);
            if !has_directory && *dirp as c_int == NUL && !reading {
                let mut ret: c_int = 0;
                let mut failed_dir: *mut c_char = ptr::null_mut();
                ret = os_mkdir_recurse(
                    &raw mut dir_name as *mut c_char,
                    0o755 as int32_t,
                    &raw mut failed_dir,
                    ptr::null_mut(),
                );
                if ret != 0 {
                    semsg_c!(
                        gettext(
                            c"E5003: Unable to create directory \"%s\" for undo file: %s".as_ptr(),
                        ),
                        failed_dir,
                        uv_strerror(ret),
                    );
                    xfree(failed_dir as *mut c_void);
                } else {
                    has_directory = true;
                }
            }
            if has_directory {
                if munged_name.is_null() {
                    munged_name = xstrdup(ffname);
                    let mut c: *mut c_char = munged_name;
                    while *c as c_int != NUL {
                        if vim_ispathsep(*c as c_int) {
                            *c = '%' as c_char;
                        }
                        c = c.offset(utfc_ptr2len(c) as isize);
                    }
                }
                undo_file_name = concat_fnames(&raw mut dir_name as *mut c_char, munged_name, true);
            }
        }
        if !undo_file_name.is_null() && (!reading || os_path_exists(undo_file_name)) {
            break;
        }
        xfree(undo_file_name.cast());
        undo_file_name = ptr::null_mut();
    }
    xfree(munged_name as *mut c_void);
    undo_file_name
}
pub(crate) unsafe fn corruption_error(mesg: *const c_char, file_name: *const c_char) {
    semsg_c!(
        gettext(c"E825: Corrupted undo file (%s): %s".as_ptr()),
        mesg,
        file_name,
    );
}
pub(crate) unsafe fn u_free_uhp(mut uhp: *mut u_header_T) {
    let mut uep: *mut u_entry_T = (*uhp).uh_entry;
    while !uep.is_null() {
        let mut nuep: *mut u_entry_T = (*uep).ue_next;
        u_freeentry(uep, (*uep).ue_size as c_int);
        uep = nuep;
    }
    xfree(uhp as *mut c_void);
}
pub(crate) unsafe fn serialize_header(mut bi: *mut bufinfo_T, mut hash: *mut uint8_t) -> bool {
    let mut buf: *mut buf_T = (*bi).bi_buf;
    let mut fp: *mut FILE = (*bi).bi_fp;
    if fwrite(
        UF_START_MAGIC.as_ptr() as *const c_void,
        UF_START_MAGIC_LEN as size_t,
        1,
        fp,
    ) != 1
    {
        return false;
    }
    undo_write_bytes(bi, UF_VERSION as uintmax_t, 2);
    if !undo_write(bi, hash, UNDO_HASH_SIZE as c_int as size_t) {
        return false;
    }
    undo_write_bytes(bi, (*buf).b_ml.ml_line_count as uintmax_t, 4);
    let mut len: size_t = if !(*buf).b_u_line_ptr.is_null() {
        strlen((*buf).b_u_line_ptr)
    } else {
        0
    };
    undo_write_bytes(bi, len as uintmax_t, 4);
    if len > 0 && !undo_write(bi, (*buf).b_u_line_ptr as *mut uint8_t, len) {
        return false;
    }
    undo_write_bytes(bi, (*buf).b_u_line_lnum as uintmax_t, 4);
    undo_write_bytes(bi, (*buf).b_u_line_colnr as uintmax_t, 4);
    put_header_ptr(bi, (*buf).b_u_oldhead);
    put_header_ptr(bi, (*buf).b_u_newhead);
    put_header_ptr(bi, (*buf).b_u_curhead);
    undo_write_bytes(bi, (*buf).b_u_numhead as uintmax_t, 4);
    undo_write_bytes(bi, (*buf).b_u_seq_last as uintmax_t, 4);
    undo_write_bytes(bi, (*buf).b_u_seq_cur as uintmax_t, 4);
    let mut time_buf: [uint8_t; 8] = [0; 8];
    time_to_bytes((*buf).b_u_time_cur, &raw mut time_buf as *mut uint8_t);
    undo_write(
        bi,
        &raw mut time_buf as *mut uint8_t,
        size_of::<[uint8_t; 8]>(),
    );
    undo_write_bytes(bi, 4, 1);
    undo_write_bytes(bi, UF_LAST_SAVE_NR as uintmax_t, 1);
    undo_write_bytes(bi, (*buf).b_u_save_nr_last as uintmax_t, 4);
    undo_write_bytes(bi, 0, 1);
    true
}
pub(crate) unsafe fn serialize_uhp(mut bi: *mut bufinfo_T, mut uhp: *mut u_header_T) -> bool {
    if !undo_write_bytes(bi, UF_HEADER_MAGIC as uintmax_t, 2) {
        return false;
    }
    put_header_ptr(bi, (*uhp).uh_next.ptr);
    put_header_ptr(bi, (*uhp).uh_prev.ptr);
    put_header_ptr(bi, (*uhp).uh_alt_next.ptr);
    put_header_ptr(bi, (*uhp).uh_alt_prev.ptr);
    undo_write_bytes(bi, (*uhp).uh_seq as uintmax_t, 4);
    serialize_pos(bi, (*uhp).uh_cursor);
    undo_write_bytes(bi, (*uhp).uh_cursor_vcol as uintmax_t, 4);
    undo_write_bytes(bi, (*uhp).uh_flags as uintmax_t, 2);
    let mut i: size_t = 0;
    while i < NMARKS as size_t {
        serialize_pos(bi, (*uhp).uh_namedm[i as usize].mark);
        i = i.wrapping_add(1);
    }
    serialize_visualinfo(bi, &raw mut (*uhp).uh_visual);
    let mut time_buf: [uint8_t; 8] = [0; 8];
    time_to_bytes((*uhp).uh_time, &raw mut time_buf as *mut uint8_t);
    undo_write(
        bi,
        &raw mut time_buf as *mut uint8_t,
        size_of::<[uint8_t; 8]>(),
    );
    undo_write_bytes(bi, 4, 1);
    undo_write_bytes(bi, UHP_SAVE_NR as uintmax_t, 1);
    undo_write_bytes(bi, (*uhp).uh_save_nr as uintmax_t, 4);
    undo_write_bytes(bi, 0, 1);
    let mut uep: *mut u_entry_T = (*uhp).uh_entry;
    while !uep.is_null() {
        undo_write_bytes(bi, UF_ENTRY_MAGIC as uintmax_t, 2);
        if !serialize_uep(bi, uep) {
            return false;
        }
        uep = (*uep).ue_next;
    }
    undo_write_bytes(bi, UF_ENTRY_END_MAGIC as uintmax_t, 2);
    let mut i_0: size_t = 0;
    while i_0 < (*uhp).uh_extmark.size {
        if !serialize_extmark(bi, *(*uhp).uh_extmark.items.add(i_0)) {
            return false;
        }
        i_0 = i_0.wrapping_add(1);
    }
    undo_write_bytes(bi, UF_ENTRY_END_MAGIC as uintmax_t, 2);
    true
}
pub(crate) unsafe fn unserialize_uhp(
    mut bi: *mut bufinfo_T,
    mut file_name: *const c_char,
) -> *mut u_header_T {
    let mut uhp: *mut u_header_T = xmalloc(size_of::<u_header_T>()) as *mut u_header_T;
    memset(uhp as *mut c_void, 0, size_of::<u_header_T>());
    (*uhp).uh_next.seq = undo_read_4c(bi);
    (*uhp).uh_prev.seq = undo_read_4c(bi);
    (*uhp).uh_alt_next.seq = undo_read_4c(bi);
    (*uhp).uh_alt_prev.seq = undo_read_4c(bi);
    (*uhp).uh_seq = undo_read_4c(bi);
    if (*uhp).uh_seq <= 0 {
        corruption_error(c"uh_seq".as_ptr(), file_name);
        xfree(uhp as *mut c_void);
        return ptr::null_mut();
    }
    unserialize_pos(bi, &raw mut (*uhp).uh_cursor);
    (*uhp).uh_cursor_vcol = undo_read_4c(bi) as colnr_T;
    (*uhp).uh_flags = undo_read_2c(bi);
    let cur_timestamp: Timestamp = os_time();
    let mut i: size_t = 0;
    while i < NMARKS as size_t {
        unserialize_pos(
            bi,
            &raw mut (*(&raw mut (*uhp).uh_namedm as *mut fmark_T).add(i)).mark,
        );
        (*uhp).uh_namedm[i as usize].timestamp = cur_timestamp;
        (*uhp).uh_namedm[i as usize].fnum = 0;
        i = i.wrapping_add(1);
    }
    unserialize_visualinfo(bi, &raw mut (*uhp).uh_visual);
    (*uhp).uh_time = undo_read_time(bi);
    loop {
        let mut len: c_int = undo_read_byte(bi);
        if len == EOF {
            corruption_error(c"truncated".as_ptr(), file_name);
            u_free_uhp(uhp);
            return ptr::null_mut();
        }
        if len == 0 {
            break;
        }
        let mut what: c_int = undo_read_byte(bi);
        match what {
            UHP_SAVE_NR => {
                (*uhp).uh_save_nr = undo_read_4c(bi);
            }
            _ => loop {
                len -= 1;
                if len < 0 {
                    break;
                }
                undo_read_byte(bi);
            },
        }
    }
    let mut last_uep: *mut u_entry_T = ptr::null_mut();
    let mut c: c_int = 0;
    loop {
        c = undo_read_2c(bi);
        if c != UF_ENTRY_MAGIC {
            break;
        }
        let mut error: bool = false;
        let mut uep: *mut u_entry_T = unserialize_uep(bi, &raw mut error, file_name);
        if last_uep.is_null() {
            (*uhp).uh_entry = uep;
        } else {
            (*last_uep).ue_next = uep;
        }
        last_uep = uep;
        if uep.is_null() || error {
            u_free_uhp(uhp);
            return ptr::null_mut();
        }
    }
    if c != UF_ENTRY_END_MAGIC {
        corruption_error(c"entry end".as_ptr(), file_name);
        u_free_uhp(uhp);
        return ptr::null_mut();
    }
    (*uhp).uh_extmark.capacity = 0;
    (*uhp).uh_extmark.size = (*uhp).uh_extmark.capacity;
    (*uhp).uh_extmark.items = ptr::null_mut();
    loop {
        c = undo_read_2c(bi);
        if c != UF_ENTRY_MAGIC {
            break;
        }
        let mut error_0: bool = false;
        let mut extup: *mut ExtmarkUndoObject =
            unserialize_extmark(bi, &raw mut error_0, file_name);
        if error_0 {
            xfree((*uhp).uh_extmark.items as *mut c_void);
            (*uhp).uh_extmark.capacity = 0;
            (*uhp).uh_extmark.size = (*uhp).uh_extmark.capacity;
            (*uhp).uh_extmark.items = ptr::null_mut();
            xfree(extup as *mut c_void);
            return ptr::null_mut();
        }
        if (*uhp).uh_extmark.size == (*uhp).uh_extmark.capacity {
            (*uhp).uh_extmark.capacity = if (*uhp).uh_extmark.capacity != 0 {
                (*uhp).uh_extmark.capacity << 1
            } else {
                8
            };
            (*uhp).uh_extmark.items = xrealloc(
                (*uhp).uh_extmark.items as *mut c_void,
                size_of::<ExtmarkUndoObject>().wrapping_mul((*uhp).uh_extmark.capacity),
            ) as *mut ExtmarkUndoObject;
        };
        let c2rust_fresh3 = (*uhp).uh_extmark.size;
        (*uhp).uh_extmark.size = (*uhp).uh_extmark.size.wrapping_add(1);
        *(*uhp).uh_extmark.items.add(c2rust_fresh3) = *extup;
        xfree(extup as *mut c_void);
    }
    if c != UF_ENTRY_END_MAGIC {
        corruption_error(c"entry end".as_ptr(), file_name);
        u_free_uhp(uhp);
        return ptr::null_mut();
    }
    uhp
}
pub(crate) unsafe fn serialize_extmark(
    mut bi: *mut bufinfo_T,
    mut extup: ExtmarkUndoObject,
) -> bool {
    if extup.type_0 as c_uint == kExtmarkSplice as c_int as c_uint {
        undo_write_bytes(bi, UF_ENTRY_MAGIC as uintmax_t, 2);
        undo_write_bytes(bi, extup.type_0 as uintmax_t, 4);
        if !undo_write(
            bi,
            &raw mut extup.data.splice as *mut uint8_t,
            size_of::<ExtmarkSplice>(),
        ) {
            return false;
        }
    } else if extup.type_0 as c_uint == kExtmarkMove as c_int as c_uint {
        undo_write_bytes(bi, UF_ENTRY_MAGIC as uintmax_t, 2);
        undo_write_bytes(bi, extup.type_0 as uintmax_t, 4);
        if !undo_write(
            bi,
            &raw mut extup.data.move_0 as *mut uint8_t,
            size_of::<ExtmarkMove>(),
        ) {
            return false;
        }
    }
    true
}
pub(crate) unsafe fn unserialize_extmark(
    mut bi: *mut bufinfo_T,
    mut error: *mut bool,
    mut _filename: *const c_char,
) -> *mut ExtmarkUndoObject {
    let mut buf: *mut uint8_t = ptr::null_mut();
    let mut extup: *mut ExtmarkUndoObject =
        xmalloc(size_of::<ExtmarkUndoObject>()) as *mut ExtmarkUndoObject;
    let mut type_0: UndoObjectType = undo_read_4c(bi) as UndoObjectType;
    (*extup).type_0 = type_0;
    '_error: {
        if type_0 as c_uint == kExtmarkSplice as c_int as c_uint {
            let mut n_elems: size_t = size_of::<ExtmarkSplice>().wrapping_div(size_of::<uint8_t>());
            buf = xcalloc(n_elems, size_of::<uint8_t>()) as *mut uint8_t;
            if !undo_read(bi, buf, n_elems) {
                break '_error;
            } else {
                (*extup).data.splice = *(buf as *mut ExtmarkSplice);
            }
        } else if type_0 as c_uint == kExtmarkMove as c_int as c_uint {
            let mut n_elems_0: size_t = size_of::<ExtmarkMove>().wrapping_div(size_of::<uint8_t>());
            buf = xcalloc(n_elems_0, size_of::<uint8_t>()) as *mut uint8_t;
            if !undo_read(bi, buf, n_elems_0) {
                break '_error;
            } else {
                (*extup).data.move_0 = *(buf as *mut ExtmarkMove);
            }
        } else {
            break '_error;
        }
        xfree(buf as *mut c_void);
        return extup;
    }
    xfree(extup as *mut c_void);
    if !buf.is_null() {
        xfree(buf as *mut c_void);
    }
    *error = true;
    ptr::null_mut()
}
pub(crate) unsafe fn serialize_uep(mut bi: *mut bufinfo_T, mut uep: *mut u_entry_T) -> bool {
    undo_write_bytes(bi, (*uep).ue_top as uintmax_t, 4);
    undo_write_bytes(bi, (*uep).ue_bot as uintmax_t, 4);
    undo_write_bytes(bi, (*uep).ue_lcount as uintmax_t, 4);
    undo_write_bytes(bi, (*uep).ue_size as uintmax_t, 4);
    let mut i: size_t = 0;
    while i < (*uep).ue_size as size_t {
        let mut len: size_t = strlen(*(*uep).ue_array.add(i));
        if !undo_write_bytes(bi, len as uintmax_t, 4) {
            return false;
        }
        if len > 0 && !undo_write(bi, *(*uep).ue_array.add(i) as *mut uint8_t, len) {
            return false;
        }
        i = i.wrapping_add(1);
    }
    true
}
pub(crate) unsafe fn unserialize_uep(
    mut bi: *mut bufinfo_T,
    mut error: *mut bool,
    mut file_name: *const c_char,
) -> *mut u_entry_T {
    let mut uep: *mut u_entry_T = xmalloc(size_of::<u_entry_T>()) as *mut u_entry_T;
    memset(uep as *mut c_void, 0, size_of::<u_entry_T>());
    (*uep).ue_top = undo_read_4c(bi) as linenr_T;
    (*uep).ue_bot = undo_read_4c(bi) as linenr_T;
    (*uep).ue_lcount = undo_read_4c(bi) as linenr_T;
    (*uep).ue_size = undo_read_4c(bi) as linenr_T;
    let mut array: *mut *mut c_char = ptr::null_mut();
    if (*uep).ue_size > 0
        && ((*uep).ue_size as size_t) < (SIZE_MAX as usize).wrapping_div(size_of::<*mut c_char>())
    {
        array = xmalloc(size_of::<*mut c_char>().wrapping_mul((*uep).ue_size as size_t))
            as *mut *mut c_char;
        memset(
            array as *mut c_void,
            0,
            size_of::<*mut c_char>().wrapping_mul((*uep).ue_size as size_t),
        );
    }
    (*uep).ue_array = array;
    let mut i: size_t = 0;
    while i < (*uep).ue_size as size_t {
        let mut line_len: c_int = undo_read_4c(bi);
        let mut line: *mut c_char = ptr::null_mut();
        if line_len >= 0 {
            line = undo_read_string(bi, line_len as size_t);
        } else {
            line = ptr::null_mut();
            corruption_error(c"line length".as_ptr(), file_name);
        }
        if line.is_null() {
            *error = true;
            return uep;
        }
        *array.add(i) = line;
        i = i.wrapping_add(1);
    }
    uep
}
pub(crate) unsafe fn serialize_pos(mut bi: *mut bufinfo_T, mut pos: pos_T) {
    undo_write_bytes(bi, pos.lnum as uintmax_t, 4);
    undo_write_bytes(bi, pos.col as uintmax_t, 4);
    undo_write_bytes(bi, pos.coladd as uintmax_t, 4);
}
pub(crate) unsafe fn unserialize_pos(mut bi: *mut bufinfo_T, mut pos: *mut pos_T) {
    (*pos).lnum = undo_read_4c(bi) as linenr_T;
    (*pos).lnum = if (*pos).lnum > 0 { (*pos).lnum } else { 0 };
    (*pos).col = undo_read_4c(bi) as colnr_T;
    (*pos).col = (if (*pos).col > 0 {
        (*pos).col as c_int
    } else {
        0
    }) as colnr_T;
    (*pos).coladd = undo_read_4c(bi) as colnr_T;
    (*pos).coladd = (if (*pos).coladd > 0 {
        (*pos).coladd as c_int
    } else {
        0
    }) as colnr_T;
}
pub(crate) unsafe fn serialize_visualinfo(mut bi: *mut bufinfo_T, mut info: *mut visualinfo_T) {
    serialize_pos(bi, (*info).vi_start);
    serialize_pos(bi, (*info).vi_end);
    undo_write_bytes(bi, (*info).vi_mode as uintmax_t, 4);
    undo_write_bytes(bi, (*info).vi_curswant as uintmax_t, 4);
}
pub(crate) unsafe fn unserialize_visualinfo(mut bi: *mut bufinfo_T, mut info: *mut visualinfo_T) {
    unserialize_pos(bi, &raw mut (*info).vi_start);
    unserialize_pos(bi, &raw mut (*info).vi_end);
    (*info).vi_mode = undo_read_4c(bi);
    (*info).vi_curswant = undo_read_4c(bi) as colnr_T;
}
pub(crate) unsafe fn undo_write(
    mut bi: *mut bufinfo_T,
    mut ptr: *mut uint8_t,
    mut len: size_t,
) -> bool {
    fwrite(ptr as *const c_void, len, 1, (*bi).bi_fp) == 1
}
/// Writes `nr` as a `len`-byte big-endian field. See [`encode_be`].
pub(crate) unsafe fn undo_write_bytes(bi: *mut bufinfo_T, nr: uintmax_t, len: size_t) -> bool {
    let mut buf = encode_be(nr, len);
    undo_write(bi, buf.as_mut_ptr(), len)
}
pub(crate) unsafe fn put_header_ptr(mut bi: *mut bufinfo_T, mut uhp: *mut u_header_T) {
    debug_assert!(
        uhp.is_null() || (*uhp).uh_seq >= 0,
        "uhp == NULL || uhp->uh_seq >= 0"
    );
    undo_write_bytes(
        bi,
        (if !uhp.is_null() { (*uhp).uh_seq } else { 0 }) as uintmax_t,
        4,
    );
}
pub(crate) unsafe fn undo_read_4c(mut bi: *mut bufinfo_T) -> c_int {
    get4c((*bi).bi_fp)
}
pub(crate) unsafe fn undo_read_2c(mut bi: *mut bufinfo_T) -> c_int {
    get2c((*bi).bi_fp)
}
pub(crate) unsafe fn undo_read_byte(mut bi: *mut bufinfo_T) -> c_int {
    getc((*bi).bi_fp)
}
pub(crate) unsafe fn undo_read_time(mut bi: *mut bufinfo_T) -> time_t {
    get8ctime((*bi).bi_fp)
}
pub(crate) unsafe fn undo_read(
    mut bi: *mut bufinfo_T,
    mut buffer: *mut uint8_t,
    mut size: size_t,
) -> bool {
    let retval: bool = fread(buffer as *mut c_void, size, 1, (*bi).bi_fp) == 1;
    if !retval {
        memset(buffer as *mut c_void, 0, size);
    }
    retval
}
pub(crate) unsafe fn undo_read_string(mut bi: *mut bufinfo_T, mut len: size_t) -> *mut c_char {
    let mut ptr: *mut c_char = xmallocz(len) as *mut c_char;
    if len > 0 && !undo_read(bi, ptr as *mut uint8_t, len) {
        xfree(ptr as *mut c_void);
        return ptr::null_mut();
    }
    ptr
}
