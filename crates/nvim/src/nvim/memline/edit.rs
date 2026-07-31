//! Inserting and deleting a line: the two operations that change
//! the tree's shape.
//!
//! Both are one long function because both have to handle the block splitting
//! and merging inline: `ml_append_int` when a data block overflows (up to
//! three-way, and then a pointer block above it may overflow too), and
//! `ml_delete_int` when a data block empties.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn ml_append_int(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut line_arg: *mut ::core::ffi::c_char,
    mut len_arg: colnr_T,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut db_idx: ::core::ffi::c_int = 0;
        let mut line_count: ::core::ffi::c_int = 0;
        let mut dp: *mut DataBlock = ::core::ptr::null_mut::<DataBlock>();
        let mut line: *mut ::core::ffi::c_char = line_arg;
        let mut len: colnr_T = len_arg;
        if lnum > (*buf).b_ml.ml_line_count || (*buf).b_ml.ml_mfp.is_null() {
            return FAIL;
        }
        if lowest_marked.get() != 0 && lowest_marked.get() > lnum {
            lowest_marked.set(lnum + 1 as linenr_T);
        }
        if len == 0 as ::core::ffi::c_int {
            len = (strlen(line) as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as colnr_T;
        }
        let mut space_needed: int64_t = len as int64_t + INDEX_SIZE as int64_t;
        let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
        let mut page_size: int64_t = (*mfp).mf_page_size as int64_t;
        let mut ret: ::core::ffi::c_int = FAIL;
        let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
        '_theend: {
            hp = ml_find_line(
                buf,
                if lnum == 0 as linenr_T {
                    1 as linenr_T
                } else {
                    lnum
                },
                ML_INSERT as ::core::ffi::c_int,
            );
            if !hp.is_null() {
                (*buf).b_ml.ml_flags &= !ML_EMPTY;
                db_idx = 0;
                if lnum == 0 as linenr_T {
                    db_idx = -1 as ::core::ffi::c_int;
                } else {
                    db_idx = (lnum - (*buf).b_ml.ml_locked_low) as ::core::ffi::c_int;
                }
                line_count = (*buf).b_ml.ml_locked_high as ::core::ffi::c_int
                    - (*buf).b_ml.ml_locked_low as ::core::ffi::c_int;
                dp = (*hp).bh_data as *mut DataBlock;
                if ((*dp).db_free as int64_t) < space_needed
                    && db_idx == line_count - 1 as ::core::ffi::c_int
                    && lnum < (*buf).b_ml.ml_line_count
                {
                    (*buf).b_ml.ml_locked_lineadd -= 1;
                    (*buf).b_ml.ml_locked_high -= 1;
                    hp = ml_find_line(buf, lnum + 1 as linenr_T, ML_INSERT as ::core::ffi::c_int);
                    if hp.is_null() {
                        break '_theend;
                    } else {
                        db_idx = -1 as ::core::ffi::c_int;
                        line_count = ((*buf).b_ml.ml_locked_high - (*buf).b_ml.ml_locked_low)
                            as ::core::ffi::c_int;
                        dp = (*hp).bh_data as *mut DataBlock;
                    }
                }
                if (*buf).b_prev_line_count == 0 as ::core::ffi::c_int {
                    (*buf).b_prev_line_count = (*buf).b_ml.ml_line_count as ::core::ffi::c_int;
                }
                (*buf).b_ml.ml_line_count += 1;
                if (*dp).db_free as int64_t >= space_needed {
                    (*dp).db_txt_start =
                        (*dp).db_txt_start.wrapping_sub(len as ::core::ffi::c_uint);
                    (*dp).db_free = (*dp)
                        .db_free
                        .wrapping_sub(space_needed as ::core::ffi::c_uint);
                    (*dp).db_line_count += 1;
                    if line_count > db_idx + 1 as ::core::ffi::c_int {
                        let mut offset: ::core::ffi::c_int = if db_idx < 0 as ::core::ffi::c_int {
                            (*dp).db_txt_end as ::core::ffi::c_int
                        } else {
                            (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                                .offset(db_idx as isize)
                                & DB_INDEX_MASK) as ::core::ffi::c_int
                        };
                        memmove(
                            (dp as *mut ::core::ffi::c_char).offset((*dp).db_txt_start as isize)
                                as *mut ::core::ffi::c_void,
                            (dp as *mut ::core::ffi::c_char)
                                .offset((*dp).db_txt_start as isize)
                                .offset(len as isize)
                                as *const ::core::ffi::c_void,
                            (offset as size_t).wrapping_sub(
                                ((*dp).db_txt_start as size_t).wrapping_add(len as size_t),
                            ),
                        );
                        let mut i: ::core::ffi::c_int = line_count - 1 as ::core::ffi::c_int;
                        while i > db_idx {
                            *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                                .offset((i + 1 as ::core::ffi::c_int) as isize) =
                                (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                                    .offset(i as isize))
                                .wrapping_sub(len as ::core::ffi::c_uint);
                            i -= 1;
                        }
                        *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                            .offset((db_idx + 1 as ::core::ffi::c_int) as isize) =
                            (offset as colnr_T - len) as ::core::ffi::c_uint;
                    } else {
                        *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                            .offset((db_idx + 1 as ::core::ffi::c_int) as isize) =
                            (*dp).db_txt_start;
                    }
                    memmove(
                        (dp as *mut ::core::ffi::c_char).offset(
                            *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                                .offset((db_idx + 1 as ::core::ffi::c_int) as isize)
                                as isize,
                        ) as *mut ::core::ffi::c_void,
                        line as *const ::core::ffi::c_void,
                        len as size_t,
                    );
                    if flags & ML_APPEND_MARK as ::core::ffi::c_int != 0 {
                        *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                            .offset((db_idx + 1 as ::core::ffi::c_int) as isize) |= DB_MARKED;
                    }
                    (*buf).b_ml.ml_flags |= ML_LOCKED_DIRTY;
                    if flags & ML_APPEND_NEW as ::core::ffi::c_int == 0 {
                        (*buf).b_ml.ml_flags |= ML_LOCKED_POS;
                    }
                } else {
                    let mut line_count_left: ::core::ffi::c_int = 0;
                    let mut line_count_right: ::core::ffi::c_int = 0;
                    let mut page_count_left: ::core::ffi::c_int = 0;
                    let mut page_count_right: ::core::ffi::c_int = 0;
                    let mut hp_left: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
                    let mut hp_right: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
                    let mut hp_new: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
                    let mut lines_moved: ::core::ffi::c_int = 0;
                    let mut data_moved: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut total_moved: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut stack_idx: ::core::ffi::c_int = 0;
                    let mut in_left: bool = false;
                    let mut lnum_left: linenr_T = 0;
                    let mut lnum_right: linenr_T = 0;
                    let mut pp_new: *mut PointerBlock = ::core::ptr::null_mut::<PointerBlock>();
                    if db_idx < 0 as ::core::ffi::c_int {
                        lines_moved = 0 as ::core::ffi::c_int;
                        in_left = true_0 != 0;
                    } else {
                        lines_moved = line_count - db_idx - 1 as ::core::ffi::c_int;
                        if lines_moved == 0 as ::core::ffi::c_int {
                            in_left = false_0 != 0;
                        } else {
                            data_moved = (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                                .offset(db_idx as isize)
                                & DB_INDEX_MASK)
                                .wrapping_sub((*dp).db_txt_start)
                                as ::core::ffi::c_int;
                            total_moved =
                                data_moved + lines_moved * INDEX_SIZE as ::core::ffi::c_int;
                            if (*dp).db_free as int64_t + total_moved as int64_t >= space_needed {
                                in_left = true_0 != 0;
                                space_needed = total_moved as int64_t;
                            } else {
                                in_left = false_0 != 0;
                                space_needed += total_moved as int64_t;
                            }
                        }
                    }
                    let mut page_count: int64_t =
                        (space_needed + HEADER_SIZE as int64_t + page_size - 1 as int64_t)
                            / page_size;
                    hp_new = ml_new_data(
                        mfp,
                        flags & ML_APPEND_NEW as ::core::ffi::c_int != 0,
                        page_count,
                    );
                    if db_idx < 0 as ::core::ffi::c_int {
                        hp_left = hp_new;
                        hp_right = hp;
                        line_count_left = 0 as ::core::ffi::c_int;
                        line_count_right = line_count;
                    } else {
                        hp_left = hp;
                        hp_right = hp_new;
                        line_count_left = line_count;
                        line_count_right = 0 as ::core::ffi::c_int;
                    }
                    let mut dp_right: *mut DataBlock = (*hp_right).bh_data as *mut DataBlock;
                    let mut dp_left: *mut DataBlock = (*hp_left).bh_data as *mut DataBlock;
                    let mut bnum_left: blocknr_T = (*hp_left).bh_bnum;
                    let mut bnum_right: blocknr_T = (*hp_right).bh_bnum;
                    page_count_left = (*hp_left).bh_page_count as ::core::ffi::c_int;
                    page_count_right = (*hp_right).bh_page_count as ::core::ffi::c_int;
                    if !in_left {
                        (*dp_right).db_txt_start = (*dp_right)
                            .db_txt_start
                            .wrapping_sub(len as ::core::ffi::c_uint);
                        (*dp_right).db_free = (*dp_right).db_free.wrapping_sub(
                            (len as ::core::ffi::c_uint)
                                .wrapping_add(INDEX_SIZE as ::core::ffi::c_uint),
                        );
                        *(&raw mut (*dp_right).db_index as *mut ::core::ffi::c_uint)
                            .offset(0 as ::core::ffi::c_int as isize) = (*dp_right).db_txt_start;
                        if flags & ML_APPEND_MARK as ::core::ffi::c_int != 0 {
                            *(&raw mut (*dp_right).db_index as *mut ::core::ffi::c_uint)
                                .offset(0 as ::core::ffi::c_int as isize) |= DB_MARKED;
                        }
                        memmove(
                            (dp_right as *mut ::core::ffi::c_char)
                                .offset((*dp_right).db_txt_start as isize)
                                as *mut ::core::ffi::c_void,
                            line as *const ::core::ffi::c_void,
                            len as size_t,
                        );
                        line_count_right += 1;
                    }
                    if lines_moved != 0 {
                        (*dp_right).db_txt_start = (*dp_right)
                            .db_txt_start
                            .wrapping_sub(data_moved as ::core::ffi::c_uint);
                        (*dp_right).db_free = (*dp_right)
                            .db_free
                            .wrapping_sub(total_moved as ::core::ffi::c_uint);
                        memmove(
                            (dp_right as *mut ::core::ffi::c_char)
                                .offset((*dp_right).db_txt_start as isize)
                                as *mut ::core::ffi::c_void,
                            (dp_left as *mut ::core::ffi::c_char)
                                .offset((*dp_left).db_txt_start as isize)
                                as *const ::core::ffi::c_void,
                            data_moved as size_t,
                        );
                        let mut offset_0: ::core::ffi::c_int = (*dp_right)
                            .db_txt_start
                            .wrapping_sub((*dp_left).db_txt_start)
                            as ::core::ffi::c_int;
                        (*dp_left).db_txt_start = (*dp_left)
                            .db_txt_start
                            .wrapping_add(data_moved as ::core::ffi::c_uint);
                        (*dp_left).db_free = (*dp_left)
                            .db_free
                            .wrapping_add(total_moved as ::core::ffi::c_uint);
                        let mut to: ::core::ffi::c_int = line_count_right;
                        let mut from: ::core::ffi::c_int = db_idx + 1 as ::core::ffi::c_int;
                        while from < line_count_left {
                            *(&raw mut (*dp_right).db_index as *mut ::core::ffi::c_uint)
                                .offset(to as isize) = (*(&raw mut (*dp).db_index
                                as *mut ::core::ffi::c_uint)
                                .offset(from as isize))
                            .wrapping_add(offset_0 as ::core::ffi::c_uint);
                            from += 1;
                            to += 1;
                        }
                        line_count_right += lines_moved;
                        line_count_left -= lines_moved;
                    }
                    if in_left {
                        (*dp_left).db_txt_start = (*dp_left)
                            .db_txt_start
                            .wrapping_sub(len as ::core::ffi::c_uint);
                        (*dp_left).db_free = (*dp_left).db_free.wrapping_sub(
                            (len as ::core::ffi::c_uint)
                                .wrapping_add(INDEX_SIZE as ::core::ffi::c_uint),
                        );
                        *(&raw mut (*dp_left).db_index as *mut ::core::ffi::c_uint)
                            .offset(line_count_left as isize) = (*dp_left).db_txt_start;
                        if flags & ML_APPEND_MARK as ::core::ffi::c_int != 0 {
                            *(&raw mut (*dp_left).db_index as *mut ::core::ffi::c_uint)
                                .offset(line_count_left as isize) |= DB_MARKED;
                        }
                        memmove(
                            (dp_left as *mut ::core::ffi::c_char)
                                .offset((*dp_left).db_txt_start as isize)
                                as *mut ::core::ffi::c_void,
                            line as *const ::core::ffi::c_void,
                            len as size_t,
                        );
                        line_count_left += 1;
                    }
                    if db_idx < 0 as ::core::ffi::c_int {
                        lnum_left = lnum + 1 as linenr_T;
                        lnum_right = 0 as ::core::ffi::c_int as linenr_T;
                    } else {
                        lnum_left = 0 as ::core::ffi::c_int as linenr_T;
                        if in_left {
                            lnum_right = lnum + 2 as linenr_T;
                        } else {
                            lnum_right = lnum + 1 as linenr_T;
                        }
                    }
                    (*dp_left).db_line_count = line_count_left as ::core::ffi::c_long;
                    (*dp_right).db_line_count = line_count_right as ::core::ffi::c_long;
                    if lines_moved != 0 || in_left as ::core::ffi::c_int != 0 {
                        (*buf).b_ml.ml_flags |= ML_LOCKED_DIRTY;
                    }
                    if flags & ML_APPEND_NEW as ::core::ffi::c_int == 0
                        && db_idx >= 0 as ::core::ffi::c_int
                        && in_left as ::core::ffi::c_int != 0
                    {
                        (*buf).b_ml.ml_flags |= ML_LOCKED_POS;
                    }
                    mf_put(mfp, hp_new, true_0 != 0, false_0 != 0);
                    let mut lineadd: ::core::ffi::c_int = (*buf).b_ml.ml_locked_lineadd;
                    (*buf).b_ml.ml_locked_lineadd = 0 as ::core::ffi::c_int;
                    ml_find_line(buf, 0 as linenr_T, ML_FLUSH as ::core::ffi::c_int);
                    stack_idx = (*buf).b_ml.ml_stack_top - 1 as ::core::ffi::c_int;
                    while stack_idx >= 0 as ::core::ffi::c_int {
                        let mut ip: *mut infoptr_T =
                            (*buf).b_ml.ml_stack.offset(stack_idx as isize);
                        let mut pb_idx: ::core::ffi::c_int = (*ip).ip_index;
                        hp = mf_get(mfp, (*ip).ip_bnum, 1 as ::core::ffi::c_uint);
                        if hp.is_null() {
                            break '_theend;
                        }
                        let mut pp: *mut PointerBlock = (*hp).bh_data as *mut PointerBlock;
                        if (*pp).pb_id as ::core::ffi::c_int != PTR_ID as ::core::ffi::c_int {
                            iemsg(gettext(
                                (e_pointer_block_id_wrong_three.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ));
                            mf_put(mfp, hp, false_0 != 0, false_0 != 0);
                            break '_theend;
                        } else if ((*pp).pb_count as ::core::ffi::c_int)
                            < (*pp).pb_count_max as ::core::ffi::c_int
                        {
                            if (pb_idx + 1 as ::core::ffi::c_int)
                                < (*pp).pb_count as ::core::ffi::c_int
                            {
                                memmove(
                                    (&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                        .offset((pb_idx + 2 as ::core::ffi::c_int) as isize)
                                        as *mut ::core::ffi::c_void,
                                    (&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                        .offset((pb_idx + 1 as ::core::ffi::c_int) as isize)
                                        as *const ::core::ffi::c_void,
                                    (((*pp).pb_count as ::core::ffi::c_int
                                        - pb_idx
                                        - 1 as ::core::ffi::c_int)
                                        as size_t)
                                        .wrapping_mul(::core::mem::size_of::<PointerEntry>()),
                                );
                            }
                            (*pp).pb_count = (*pp).pb_count.wrapping_add(1);
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset(pb_idx as isize))
                            .pe_line_count = line_count_left as linenr_T;
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset(pb_idx as isize))
                            .pe_bnum = bnum_left;
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset(pb_idx as isize))
                            .pe_page_count = page_count_left;
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset((pb_idx + 1 as ::core::ffi::c_int) as isize))
                            .pe_line_count = line_count_right as linenr_T;
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset((pb_idx + 1 as ::core::ffi::c_int) as isize))
                            .pe_bnum = bnum_right;
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset((pb_idx + 1 as ::core::ffi::c_int) as isize))
                            .pe_page_count = page_count_right;
                            if lnum_left != 0 as linenr_T {
                                (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                    .offset(pb_idx as isize))
                                .pe_old_lnum = lnum_left;
                            }
                            if lnum_right != 0 as linenr_T {
                                (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                    .offset((pb_idx + 1 as ::core::ffi::c_int) as isize))
                                .pe_old_lnum = lnum_right;
                            }
                            mf_put(mfp, hp, true_0 != 0, false_0 != 0);
                            (*buf).b_ml.ml_stack_top = stack_idx + 1 as ::core::ffi::c_int;
                            if lineadd != 0 {
                                (*buf).b_ml.ml_stack_top -= 1;
                                ml_lineadd(buf, lineadd);
                                (*(*buf)
                                    .b_ml
                                    .ml_stack
                                    .offset((*buf).b_ml.ml_stack_top as isize))
                                .ip_high = ((*(*buf)
                                    .b_ml
                                    .ml_stack
                                    .offset((*buf).b_ml.ml_stack_top as isize))
                                .ip_high
                                    as ::core::ffi::c_int
                                    + lineadd)
                                    as linenr_T;
                                (*buf).b_ml.ml_stack_top += 1;
                            }
                            break;
                        } else {
                            loop {
                                hp_new = ml_new_ptr(mfp);
                                if hp_new.is_null() {
                                    break '_theend;
                                }
                                pp_new = (*hp_new).bh_data as *mut PointerBlock;
                                if (*hp).bh_bnum != 1 as blocknr_T {
                                    break;
                                }
                                memmove(
                                    pp_new as *mut ::core::ffi::c_void,
                                    pp as *const ::core::ffi::c_void,
                                    page_size as size_t,
                                );
                                (*pp).pb_count = 1 as uint16_t;
                                (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .pe_bnum = (*hp_new).bh_bnum;
                                (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .pe_line_count = (*buf).b_ml.ml_line_count;
                                (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .pe_old_lnum = 1 as ::core::ffi::c_int as linenr_T;
                                (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .pe_page_count = 1 as ::core::ffi::c_int;
                                mf_put(mfp, hp, true_0 != 0, false_0 != 0);
                                hp = hp_new;
                                pp = pp_new;
                                (*ip).ip_index = 0 as ::core::ffi::c_int;
                                stack_idx += 1;
                            }
                            total_moved = (*pp).pb_count as ::core::ffi::c_int
                                - pb_idx
                                - 1 as ::core::ffi::c_int;
                            if total_moved != 0 {
                                memmove(
                                    (&raw mut (*pp_new).pb_pointer as *mut PointerEntry)
                                        .offset(0 as ::core::ffi::c_int as isize)
                                        as *mut ::core::ffi::c_void,
                                    (&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                        .offset((pb_idx + 1 as ::core::ffi::c_int) as isize)
                                        as *const ::core::ffi::c_void,
                                    (total_moved as size_t)
                                        .wrapping_mul(::core::mem::size_of::<PointerEntry>()),
                                );
                                (*pp_new).pb_count = total_moved as uint16_t;
                                (*pp).pb_count = ((*pp).pb_count as ::core::ffi::c_int
                                    - (total_moved - 1 as ::core::ffi::c_int))
                                    as uint16_t;
                                (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                    .offset((pb_idx + 1 as ::core::ffi::c_int) as isize))
                                .pe_bnum = bnum_right;
                                (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                    .offset((pb_idx + 1 as ::core::ffi::c_int) as isize))
                                .pe_line_count = line_count_right as linenr_T;
                                (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                    .offset((pb_idx + 1 as ::core::ffi::c_int) as isize))
                                .pe_page_count = page_count_right;
                                if lnum_right != 0 {
                                    (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                        .offset((pb_idx + 1 as ::core::ffi::c_int) as isize))
                                    .pe_old_lnum = lnum_right;
                                }
                            } else {
                                (*pp_new).pb_count = 1 as uint16_t;
                                (*(&raw mut (*pp_new).pb_pointer as *mut PointerEntry)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .pe_bnum = bnum_right;
                                (*(&raw mut (*pp_new).pb_pointer as *mut PointerEntry)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .pe_line_count = line_count_right as linenr_T;
                                (*(&raw mut (*pp_new).pb_pointer as *mut PointerEntry)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .pe_page_count = page_count_right;
                                (*(&raw mut (*pp_new).pb_pointer as *mut PointerEntry)
                                    .offset(0 as ::core::ffi::c_int as isize))
                                .pe_old_lnum = lnum_right;
                            }
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset(pb_idx as isize))
                            .pe_bnum = bnum_left;
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset(pb_idx as isize))
                            .pe_line_count = line_count_left as linenr_T;
                            (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                .offset(pb_idx as isize))
                            .pe_page_count = page_count_left;
                            if lnum_left != 0 {
                                (*(&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                    .offset(pb_idx as isize))
                                .pe_old_lnum = lnum_left;
                            }
                            lnum_left = 0 as ::core::ffi::c_int as linenr_T;
                            lnum_right = 0 as ::core::ffi::c_int as linenr_T;
                            line_count_right = 0 as ::core::ffi::c_int;
                            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while i_0 < (*pp_new).pb_count as ::core::ffi::c_int {
                                line_count_right += (*(&raw mut (*pp_new).pb_pointer
                                    as *mut PointerEntry)
                                    .offset(i_0 as isize))
                                .pe_line_count
                                    as ::core::ffi::c_int;
                                i_0 += 1;
                            }
                            line_count_left = 0 as ::core::ffi::c_int;
                            let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while i_1 < (*pp).pb_count as ::core::ffi::c_int {
                                line_count_left += (*(&raw mut (*pp).pb_pointer
                                    as *mut PointerEntry)
                                    .offset(i_1 as isize))
                                .pe_line_count
                                    as ::core::ffi::c_int;
                                i_1 += 1;
                            }
                            bnum_left = (*hp).bh_bnum;
                            bnum_right = (*hp_new).bh_bnum;
                            page_count_left = 1 as ::core::ffi::c_int;
                            page_count_right = 1 as ::core::ffi::c_int;
                            mf_put(mfp, hp, true_0 != 0, false_0 != 0);
                            mf_put(mfp, hp_new, true_0 != 0, false_0 != 0);
                            stack_idx -= 1;
                        }
                    }
                    if stack_idx < 0 as ::core::ffi::c_int {
                        iemsg(gettext(b"E318: Updated too many blocks?\0".as_ptr()
                            as *const ::core::ffi::c_char));
                        (*buf).b_ml.ml_stack_top = 0 as ::core::ffi::c_int;
                    }
                }
                ml_updatechunk(
                    buf,
                    lnum + 1 as linenr_T,
                    len as ::core::ffi::c_int,
                    ML_CHNK_ADDLINE,
                );
                ret = OK;
            }
        }
        return ret;
    }
}

pub(crate) unsafe extern "C" fn ml_delete_int(
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if lowest_marked.get() != 0 && lowest_marked.get() > lnum {
            (*lowest_marked.ptr()) -= 1;
        }
        if (*buf).b_ml.ml_line_count == 1 as linenr_T {
            if flags & ML_DEL_MESSAGE as ::core::ffi::c_int != 0 {
                set_keep_msg(
                    gettext(no_lines_msg.ptr() as *mut ::core::ffi::c_char),
                    0 as ::core::ffi::c_int,
                );
            }
            let mut i: ::core::ffi::c_int = ml_replace_buf(
                buf,
                1 as linenr_T,
                b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                true_0 != 0,
                false_0 != 0,
            );
            (*buf).b_ml.ml_flags |= ML_EMPTY;
            return i;
        }
        let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
        if mfp.is_null() {
            return FAIL;
        }
        let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
        hp = ml_find_line(buf, lnum, ML_DELETE as ::core::ffi::c_int);
        if hp.is_null() {
            return FAIL;
        }
        let mut dp: *mut DataBlock = (*hp).bh_data as *mut DataBlock;
        let mut count: ::core::ffi::c_int = (*buf).b_ml.ml_locked_high as ::core::ffi::c_int
            - (*buf).b_ml.ml_locked_low as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int;
        let mut idx: ::core::ffi::c_int =
            lnum as ::core::ffi::c_int - (*buf).b_ml.ml_locked_low as ::core::ffi::c_int;
        if (*buf).b_prev_line_count == 0 as ::core::ffi::c_int {
            (*buf).b_prev_line_count = (*buf).b_ml.ml_line_count as ::core::ffi::c_int;
        }
        (*buf).b_ml.ml_line_count -= 1;
        let mut line_start: ::core::ffi::c_int =
            (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint).offset(idx as isize)
                & DB_INDEX_MASK) as ::core::ffi::c_int;
        let mut line_size: ::core::ffi::c_int = 0;
        if idx == 0 as ::core::ffi::c_int {
            line_size = (*dp)
                .db_txt_end
                .wrapping_sub(line_start as ::core::ffi::c_uint)
                as ::core::ffi::c_int;
        } else {
            line_size = (*(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                .offset((idx - 1 as ::core::ffi::c_int) as isize)
                & DB_INDEX_MASK)
                .wrapping_sub(line_start as ::core::ffi::c_uint)
                as ::core::ffi::c_int;
        }
        '_c2rust_label: {
            if line_size >= 1 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"line_size >= 1\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/memline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2687 as ::core::ffi::c_uint,
                    b"int ml_delete_int(buf_T *, linenr_T, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        ml_add_deleted_len_buf(
            buf,
            (dp as *mut ::core::ffi::c_char).offset(line_start as isize),
            (line_size - 1 as ::core::ffi::c_int) as ssize_t,
        );
        let mut ret: ::core::ffi::c_int = FAIL;
        '_theend: {
            's_274: {
                if count == 1 as ::core::ffi::c_int {
                    mf_free(mfp, hp);
                    (*buf).b_ml.ml_locked = ::core::ptr::null_mut::<bhdr_T>();
                    let mut stack_idx: ::core::ffi::c_int =
                        (*buf).b_ml.ml_stack_top - 1 as ::core::ffi::c_int;
                    loop {
                        if stack_idx < 0 as ::core::ffi::c_int {
                            break 's_274;
                        }
                        (*buf).b_ml.ml_stack_top = 0 as ::core::ffi::c_int;
                        let mut ip: *mut infoptr_T =
                            (*buf).b_ml.ml_stack.offset(stack_idx as isize);
                        idx = (*ip).ip_index;
                        hp = mf_get(mfp, (*ip).ip_bnum, 1 as ::core::ffi::c_uint);
                        if hp.is_null() {
                            break '_theend;
                        }
                        let mut pp: *mut PointerBlock = (*hp).bh_data as *mut PointerBlock;
                        if (*pp).pb_id as ::core::ffi::c_int != PTR_ID as ::core::ffi::c_int {
                            iemsg(gettext(
                                (e_pointer_block_id_wrong_four.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ));
                            mf_put(mfp, hp, false_0 != 0, false_0 != 0);
                            break '_theend;
                        } else {
                            (*pp).pb_count = (*pp).pb_count.wrapping_sub(1);
                            count = (*pp).pb_count as ::core::ffi::c_int;
                            if count == 0 as ::core::ffi::c_int {
                                mf_free(mfp, hp);
                                stack_idx -= 1;
                            } else {
                                if count != idx {
                                    memmove(
                                        (&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                            .offset(idx as isize)
                                            as *mut ::core::ffi::c_void,
                                        (&raw mut (*pp).pb_pointer as *mut PointerEntry)
                                            .offset((idx + 1 as ::core::ffi::c_int) as isize)
                                            as *const ::core::ffi::c_void,
                                        ((count - idx) as size_t)
                                            .wrapping_mul(::core::mem::size_of::<PointerEntry>()),
                                    );
                                }
                                mf_put(mfp, hp, true_0 != 0, false_0 != 0);
                                (*buf).b_ml.ml_stack_top = stack_idx;
                                if (*buf).b_ml.ml_locked_lineadd != 0 as ::core::ffi::c_int {
                                    ml_lineadd(buf, (*buf).b_ml.ml_locked_lineadd);
                                    (*(*buf)
                                        .b_ml
                                        .ml_stack
                                        .offset((*buf).b_ml.ml_stack_top as isize))
                                    .ip_high = ((*(*buf)
                                        .b_ml
                                        .ml_stack
                                        .offset((*buf).b_ml.ml_stack_top as isize))
                                    .ip_high
                                        as ::core::ffi::c_int
                                        + (*buf).b_ml.ml_locked_lineadd)
                                        as linenr_T;
                                }
                                (*buf).b_ml.ml_stack_top += 1;
                                break 's_274;
                            }
                        }
                    }
                } else {
                    let mut text_start: ::core::ffi::c_int =
                        (*dp).db_txt_start as ::core::ffi::c_int;
                    memmove(
                        (dp as *mut ::core::ffi::c_char)
                            .offset(text_start as isize)
                            .offset(line_size as isize)
                            as *mut ::core::ffi::c_void,
                        (dp as *mut ::core::ffi::c_char).offset(text_start as isize)
                            as *const ::core::ffi::c_void,
                        (line_start - text_start) as size_t,
                    );
                    let mut i_0: ::core::ffi::c_int = idx;
                    while i_0 < count - 1 as ::core::ffi::c_int {
                        *(&raw mut (*dp).db_index as *mut ::core::ffi::c_uint)
                            .offset(i_0 as isize) = (*(&raw mut (*dp).db_index
                            as *mut ::core::ffi::c_uint)
                            .offset((i_0 + 1 as ::core::ffi::c_int) as isize))
                        .wrapping_add(line_size as ::core::ffi::c_uint);
                        i_0 += 1;
                    }
                    (*dp).db_free = (*dp).db_free.wrapping_add(
                        (line_size as ::core::ffi::c_uint)
                            .wrapping_add(INDEX_SIZE as ::core::ffi::c_uint),
                    );
                    (*dp).db_txt_start = (*dp)
                        .db_txt_start
                        .wrapping_add(line_size as ::core::ffi::c_uint);
                    (*dp).db_line_count -= 1;
                    (*buf).b_ml.ml_flags |= ML_LOCKED_DIRTY | ML_LOCKED_POS;
                }
            }
            ml_updatechunk(buf, lnum, line_size, ML_CHNK_DELLINE);
            ret = OK;
        }
        return ret;
    }
}
