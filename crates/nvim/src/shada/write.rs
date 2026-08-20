//! Writing a ShaDa file.
//!
//! `shada_write` is the whole of it: work out what the `'shada'` option
//! allows, merge in what the old file held (see `merge`), collect the
//! editor's state (see `collect`), and pack the result (see `pack`) in the
//! order the format wants.
//!
//! The collecting and the packing are two separate passes because of the
//! merge in between: everything is gathered into a [`WriteMergerState`],
//! the old file is read over it so that the newer of each pair wins, and
//! only then is the result written out.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint, c_void};

use super::*;
use crate::types::builders::{DictBuf, static_cstring};
use crate::types::{NUL, Object, VAR_DICT, VAR_FLAVOUR_SHADA, VAR_FUNC, VAR_LIST, VAR_PARTIAL};

/// What the `'shada'` option allows a write to contain.
struct Limits {
    /// `s`: entries whose payload is bigger than this many kilobytes are
    /// dropped. Zero means nothing is written at all.
    max_kbyte: size_t,
    /// `<`, or `"` when `<` is absent: how many lines of a register are
    /// kept. Zero means no registers.
    max_reg_lines: c_int,
    /// `'`: how many files' local marks are kept. Zero also turns off the
    /// jump list and the change lists.
    num_marked_files: size_t,
    /// `!`: whether global variables are written.
    global_vars: bool,
    /// `f`: whether global and numbered marks are written.
    global_marks: bool,
}

impl Limits {
    /// Read `'shada'`. `None` when `s0` says the file may hold nothing.
    unsafe fn from_shada_option() -> Option<Limits> {
        unsafe {
            let mut max_kbyte_i = get_shada_parameter('s' as c_int);
            if max_kbyte_i < 0 {
                // Not given: the format's own default.
                max_kbyte_i = 10;
            }
            if max_kbyte_i == 0 {
                return None;
            }
            let mut max_reg_lines = get_shada_parameter('<' as c_int);
            if max_reg_lines < 0 {
                max_reg_lines = get_shada_parameter('"' as c_int);
            }
            Some(Limits {
                max_kbyte: max_kbyte_i as size_t,
                max_reg_lines,
                num_marked_files: get_shada_parameter('\'' as c_int) as size_t,
                global_vars: !find_shada_parameter('!' as c_int).is_null(),
                global_marks: get_shada_parameter('f' as c_int) != 0,
            })
        }
    }

    /// Whether any register is written.
    fn registers(&self) -> bool {
        self.max_reg_lines != 0
    }
}

/// One ShaDa write in progress.
struct Writing {
    /// Everything collected so far, and what the merge decides between.
    /// Heap-allocated because it is some 60 KiB of mark and jump slots.
    wms: *mut WriteMergerState,
    /// Buffers whose file is somewhere marks are not kept for — a removable
    /// medium, or `'viewdir'`-style scratch.
    removable_bufs: Set_ptr_t,
    /// The output, buffered in the file's own write buffer.
    packer: PackerBuffer,
    limits: Limits,
    /// Which history types are being written.
    histories: [bool; HIST_COUNT as usize],
}

/// Write a ShaDa file, merging `sd_reader`'s contents into it when there is
/// an old file to merge.
pub(crate) unsafe fn shada_write(
    sd_writer: *mut FileDescriptor,
    sd_reader: *mut FileDescriptor,
) -> ShaDaWriteResult {
    unsafe {
        let Some(limits) = Limits::from_shada_option() else {
            return kSDWriteSuccessful;
        };

        let wms = xcalloc(1, size_of::<WriteMergerState>()).cast::<WriteMergerState>();
        let histories = init_histories(wms, !sd_reader.is_null());
        let srni_flags = wanted_kinds(&limits, &histories);

        let mut writing = Writing {
            wms,
            removable_bufs: Set_ptr_t {
                h: MAPHASH_INIT,
                keys: core::ptr::null_mut(),
            },
            packer: packer_buffer_for_file(sd_writer),
            limits,
            histories,
        };

        // Recording where the cursor is in every window is what makes the
        // `"` mark right on exit. It also means `:wshada` moves that mark
        // to the cursor, as `:wviminfo` did.
        for wp in all_windows() {
            set_last_cursor(wp);
        }
        find_removable_bufs(&raw mut writing.removable_bufs);

        let ret = writing.run(sd_reader, srni_flags);
        writing.finish();
        ret
    }
}

/// Start a history merger for each history type `'shada'` keeps something
/// of, and answer which those are.
unsafe fn init_histories(wms: *mut WriteMergerState, merging: bool) -> [bool; HIST_COUNT as usize] {
    unsafe {
        let mut wanted = [false; HIST_COUNT as usize];
        for (i, wanted) in wanted.iter_mut().enumerate() {
            let mut num_saved = get_shada_parameter(hist_type2char(i as c_int));
            if num_saved == -1 {
                num_saved = p_hi.get() as c_int;
            }
            if num_saved > 0 {
                *wanted = true;
                hms_init(
                    &raw mut (*wms).hms[i],
                    i as uint8_t,
                    num_saved as size_t,
                    merging,
                    false,
                );
            }
        }
        wanted
    }
}

/// The kinds of entry the merging read wants out of the old file. What is
/// not asked for is copied straight through rather than merged.
fn wanted_kinds(limits: &Limits, histories: &[bool; HIST_COUNT as usize]) -> c_uint {
    let mut kinds = kSDReadUndisableableData | kSDReadUnknown;
    if histories.contains(&true) {
        kinds |= kSDReadHistory;
    }
    if limits.registers() {
        kinds |= kSDReadRegisters;
    }
    if limits.global_vars {
        kinds |= kSDReadVariables;
    }
    if limits.global_marks {
        kinds |= kSDReadGlobalMarks;
    }
    if limits.num_marked_files != 0 {
        kinds |= kSDReadLocalMarks | kSDReadChanges;
    }
    kinds
}

impl Writing {
    /// Collect, merge and pack. A failure to write stops the pass; the
    /// caller still tears everything down.
    unsafe fn run(
        &mut self,
        sd_reader: *mut FileDescriptor,
        srni_flags: c_uint,
    ) -> ShaDaWriteResult {
        unsafe {
            if self.write_header() == kSDWriteFailed {
                return kSDWriteFailed;
            }
            if !find_shada_parameter('%' as c_int).is_null()
                && self.write_buflist() == kSDWriteFailed
            {
                return kSDWriteFailed;
            }
            // Variables go out as they are found rather than into `wms`;
            // only their names are kept, so that the merge knows which of
            // the old file's variables have already been written.
            if self.limits.global_vars && self.dump_variables() == kSDWriteFailed {
                return kSDWriteFailed;
            }

            self.collect_jumps();
            self.collect_search_patterns();
            self.collect_global_marks();
            if self.limits.registers() {
                shada_initialize_registers(self.wms, self.limits.max_reg_lines);
            }
            self.collect_buffer_marks();

            // Whatever the old file holds that this Nvim has no opinion
            // about, or a staler one.
            let mut ret = kSDWriteSuccessful;
            if !sd_reader.is_null() {
                let srww_ret = shada_read_when_writing(
                    sd_reader,
                    srni_flags,
                    self.limits.max_kbyte,
                    self.wms,
                    &raw mut self.packer,
                );
                if srww_ret != kSDWriteSuccessful {
                    ret = srww_ret;
                }
            }

            self.update_numbered_marks();
            if self.pack_everything() == kSDWriteFailed {
                return kSDWriteFailed;
            }
            ret
        }
    }

    /// What this Nvim was. Nothing ever reads it back; it is there for
    /// anyone looking at the file by hand.
    unsafe fn write_header(&mut self) -> ShaDaWriteResult {
        unsafe {
            let mut header = DictBuf::<5>::new();
            header
                .insert(c"generator", Object::string(static_cstring(c"nvim")))
                .insert(c"version", Object::string(static_cstring(LONG_VERSION)))
                .insert(
                    c"max_kbyte",
                    Object::integer(self.limits.max_kbyte as Integer),
                )
                .insert(c"pid", Object::integer(os_get_pid()))
                .insert(c"encoding", Object::string(cstr_as_string(p_enc.get())));
            self.pack(
                ShadaEntry {
                    type_0: kSDItemHeader,
                    can_free_entry: false,
                    timestamp: os_time(),
                    data: C2Rust_Unnamed_22 {
                        header: header.dict(),
                    },
                    additional_data: core::ptr::null_mut(),
                },
                0,
            )
        }
    }

    /// The list of files this Nvim has buffers for, so that a later start
    /// can reopen them.
    unsafe fn write_buflist(&mut self) -> ShaDaWriteResult {
        unsafe {
            let entry = shada_get_buflist(&raw mut self.removable_bufs);
            let ret = self.pack(entry, 0);
            xfree(entry.data.buffer_list.buffers.cast());
            ret
        }
    }

    /// Every global variable `'shada'` says to keep, written as it is found.
    ///
    /// A container that turns out to be part of a cycle is skipped: the
    /// encoder would not terminate on it.
    unsafe fn dump_variables(&mut self) -> ShaDaWriteResult {
        unsafe {
            let mut var_iter: *const c_void = core::ptr::null();
            let timestamp = os_time();
            loop {
                let mut vartv: typval_T = core::mem::zeroed();
                let mut name: *const c_char = core::ptr::null();
                var_iter =
                    var_shada_iter(var_iter, &raw mut name, &raw mut vartv, VAR_FLAVOUR_SHADA);
                if name.is_null() {
                    return kSDWriteSuccessful;
                }

                if !writable_value(&vartv) {
                    tv_clear(&raw mut vartv);
                    if var_iter.is_null() {
                        return kSDWriteSuccessful;
                    }
                    continue;
                }

                // The entry takes a copy, which the pack frees; the value
                // the iterator handed over is this function's to release.
                let mut tgttv: typval_T = core::mem::zeroed();
                tv_copy(&raw mut vartv, &raw mut tgttv);
                let ret = self.pack(
                    ShadaEntry {
                        type_0: kSDItemVariable,
                        can_free_entry: false,
                        timestamp,
                        data: C2Rust_Unnamed_22 {
                            global_var: global_var {
                                name: name.cast_mut(),
                                value: tgttv,
                            },
                        },
                        additional_data: core::ptr::null_mut(),
                    },
                    self.limits.max_kbyte,
                );
                tv_clear(&raw mut vartv);
                tv_clear(&raw mut tgttv);
                if ret == kSDWriteFailed {
                    return kSDWriteFailed;
                }
                if ret == kSDWriteSuccessful {
                    set_put_cstr_t(
                        &raw mut (*self.wms).dumped_variables,
                        name,
                        core::ptr::null_mut(),
                    );
                }
                if var_iter.is_null() {
                    return kSDWriteSuccessful;
                }
            }
        }
    }

    /// The jump list, as far back as `'shada'` keeps files' marks.
    unsafe fn collect_jumps(&mut self) {
        unsafe {
            if self.limits.num_marked_files > 0 {
                (*self.wms).jumps_size = shada_init_jumps(
                    (&raw mut (*self.wms).jumps).cast::<ShadaEntry>(),
                    &raw mut self.removable_bufs,
                );
            }
        }
    }

    /// The last search and substitute patterns, and the last `:substitute`
    /// replacement string. All three ride on the search history's setting.
    unsafe fn collect_search_patterns(&mut self) {
        unsafe {
            if !self.histories[HIST_SEARCH as usize] {
                return;
            }
            let highlighted = !(no_hlsearch.get() || !find_shada_parameter('h' as c_int).is_null());
            let last_used = search_was_last_used();

            add_search_pattern(
                &raw mut (*self.wms).search_pattern,
                Some(get_search_pattern),
                false,
                last_used,
                highlighted,
            );
            add_search_pattern(
                &raw mut (*self.wms).sub_search_pattern,
                Some(get_substitute_pattern),
                true,
                last_used,
                highlighted,
            );

            let mut sub: SubReplacementString = core::mem::zeroed();
            sub_get_replacement(&raw mut sub);
            // An empty replacement string is not worth storing.
            if !sub.sub.is_null() {
                (*self.wms).replacement = ShadaEntry {
                    type_0: kSDItemSubString,
                    can_free_entry: false,
                    timestamp: sub.timestamp,
                    data: C2Rust_Unnamed_22 {
                        sub_string: sub_string { sub: sub.sub },
                    },
                    additional_data: sub.additional_data,
                };
            }
        }
    }

    /// The lettered and numbered global marks.
    ///
    /// A mark on a file no buffer holds still names the file; one on a
    /// buffer that has gone, or that sits on a removable medium, is dropped.
    unsafe fn collect_global_marks(&mut self) {
        unsafe {
            if !self.limits.global_marks {
                return;
            }
            let mut mark_iter: *const c_void = core::ptr::null();
            let mut digit_mark_idx = 0;
            loop {
                let mut name: c_char = NUL as c_char;
                let mut fm: xfmark_T = core::mem::zeroed();
                mark_iter = mark_global_iter(mark_iter, &raw mut name, &raw mut fm);
                if name as c_int == NUL {
                    return;
                }

                if let Some(fname) = self.mark_fname(&fm) {
                    let entry = ShadaEntry {
                        type_0: kSDItemGlobalMark,
                        can_free_entry: false,
                        timestamp: fm.fmark.timestamp,
                        data: C2Rust_Unnamed_22 {
                            filemark: shada_filemark {
                                name,
                                mark: fm.fmark.mark,
                                fname: fname.cast_mut(),
                            },
                        },
                        additional_data: fm.fmark.additional_data,
                    };
                    // The ten numbered marks are simply the ten most
                    // recent; their names are assigned on the way out.
                    if ascii_isdigit(name as c_int) {
                        replace_numbered_mark(self.wms, digit_mark_idx, entry);
                        digit_mark_idx += 1;
                    } else {
                        (*self.wms).global_marks[mark_global_index(name) as usize] = entry;
                    }
                }

                if mark_iter.is_null() {
                    return;
                }
            }
        }
    }

    /// The file name to record a global mark against, or `None` when the
    /// mark is not worth keeping.
    unsafe fn mark_fname(&mut self, fm: &xfmark_T) -> Option<*const c_char> {
        unsafe {
            if fm.fmark.fnum == 0 {
                debug_assert!(!fm.fname.is_null(), "shada: a mark with no buffer or file");
                return (!shada_removable(fm.fname)).then_some(fm.fname);
            }
            let buf = buflist_findnr(fm.fmark.fnum);
            if buf.is_null()
                || (*buf).b_ffname.is_null()
                || set_has_ptr_t(&raw mut self.removable_bufs, buf.cast::<c_void>())
            {
                return None;
            }
            Some((*buf).b_ffname)
        }
    }

    /// Every buffer's local marks and change list, keyed by file name so
    /// that the merge can find the same file's marks in the old file.
    unsafe fn collect_buffer_marks(&mut self) {
        unsafe {
            if self.limits.num_marked_files == 0 {
                return;
            }
            let mut buf = firstbuf.get();
            while !buf.is_null() {
                if !ignore_buf(buf, &raw mut self.removable_bufs) {
                    self.collect_one_buffer(buf);
                }
                buf = (*buf).b_next;
            }
        }
    }

    unsafe fn collect_one_buffer(&mut self, buf: *mut buf_T) {
        unsafe {
            let fname = (*buf).b_ffname;
            let filemarks = self.file_marks_for(fname);

            let mut mark_iter: *const c_void = core::ptr::null();
            loop {
                let mut fm: fmark_T = core::mem::zeroed();
                let mut name: c_char = NUL as c_char;
                mark_iter = mark_buffer_iter(mark_iter, buf, &raw mut name, &raw mut fm);
                if name as c_int == NUL {
                    break;
                }
                (*filemarks).marks[mark_local_index(name) as usize] = ShadaEntry {
                    type_0: kSDItemLocalMark,
                    can_free_entry: false,
                    timestamp: fm.timestamp,
                    data: C2Rust_Unnamed_22 {
                        filemark: shada_filemark {
                            name,
                            mark: fm.mark,
                            fname,
                        },
                    },
                    additional_data: fm.additional_data,
                };
                (*filemarks).greatest_timestamp = (*filemarks).greatest_timestamp.max(fm.timestamp);
                if mark_iter.is_null() {
                    break;
                }
            }

            for i in 0..(*buf).b_changelistlen as usize {
                let fm = (*buf).b_changelist[i];
                (*filemarks).changes[i] = ShadaEntry {
                    type_0: kSDItemChange,
                    can_free_entry: false,
                    timestamp: fm.timestamp,
                    data: C2Rust_Unnamed_22 {
                        filemark: shada_filemark {
                            name: 0,
                            mark: fm.mark,
                            fname,
                        },
                    },
                    additional_data: fm.additional_data,
                };
                (*filemarks).greatest_timestamp = (*filemarks).greatest_timestamp.max(fm.timestamp);
            }
            (*filemarks).changes_size = (*buf).b_changelistlen as size_t;
        }
    }

    /// The slot one file's marks are collected into, made on first use.
    /// The map owns both its keys and its values.
    unsafe fn file_marks_for(&mut self, fname: *const c_char) -> *mut FileMarks {
        unsafe {
            let mut key_alloc: *mut cstr_t = core::ptr::null_mut();
            let mut new_item = false;
            let slot = map_put_ref_cstr_t_ptr_t(
                &raw mut (*self.wms).file_marks,
                fname,
                &raw mut key_alloc,
                &raw mut new_item,
            );
            if new_item {
                *key_alloc = xstrdup(fname);
            }
            if (*slot).is_null() {
                *slot = xcalloc(1, size_of::<FileMarks>());
            }
            (*slot).cast::<FileMarks>()
        }
    }

    /// Put the cursor's position in at `'0`, shifting the other numbered
    /// marks down and dropping `'9`.
    unsafe fn update_numbered_marks(&mut self) {
        unsafe {
            if !self.limits.global_marks
                || ignore_buf(curbuf.get(), &raw mut self.removable_bufs)
                || (*curwin.get()).w_cursor.lnum == 0
            {
                return;
            }
            replace_numbered_mark(
                self.wms,
                0,
                ShadaEntry {
                    type_0: kSDItemGlobalMark,
                    can_free_entry: false,
                    timestamp: os_time(),
                    data: C2Rust_Unnamed_22 {
                        filemark: shada_filemark {
                            name: '0' as c_char,
                            mark: (*curwin.get()).w_cursor,
                            fname: (*curbuf.get()).b_ffname,
                        },
                    },
                    additional_data: core::ptr::null_mut(),
                },
            );
        }
    }

    /// Write everything the merge left in `wms`, in the order the format
    /// wants it.
    unsafe fn pack_everything(&mut self) -> ShaDaWriteResult {
        unsafe {
            let wms = self.wms;
            if self.pack_sparse(&raw const (*wms).global_marks) == kSDWriteFailed
                || self.pack_sparse(&raw const (*wms).numbered_marks) == kSDWriteFailed
                || self.pack_sparse(&raw const (*wms).registers) == kSDWriteFailed
                || self.pack_dense((&raw const (*wms).jumps).cast(), (*wms).jumps_size)
                    == kSDWriteFailed
            {
                return kSDWriteFailed;
            }
            for entry in [
                (*wms).search_pattern,
                (*wms).sub_search_pattern,
                (*wms).replacement,
            ] {
                if entry.type_0 != kSDItemMissing && self.pack_freeing(entry) == kSDWriteFailed {
                    return kSDWriteFailed;
                }
            }
            if self.pack_file_marks() == kSDWriteFailed {
                return kSDWriteFailed;
            }
            self.pack_histories()
        }
    }

    /// Every file's marks, most recently touched file first, as many files
    /// as `'shada'` keeps.
    unsafe fn pack_file_marks(&mut self) -> ShaDaWriteResult {
        unsafe {
            let mut all: Vec<*mut FileMarks> = Vec::new();
            let marks = &(*self.wms).file_marks;
            for i in 0..marks.set.h.n_keys as usize {
                all.push((*marks.values.add(i)).cast::<FileMarks>());
            }
            qsort(
                all.as_mut_ptr().cast(),
                all.len(),
                size_of::<*mut FileMarks>(),
                Some(compare_file_marks),
            );

            let to_dump = all.len().min(self.limits.num_marked_files);
            for &file in &all[..to_dump] {
                if self.pack_sparse(&raw const (*file).marks) == kSDWriteFailed
                    || self.pack_dense((&raw const (*file).changes).cast(), (*file).changes_size)
                        == kSDWriteFailed
                {
                    return kSDWriteFailed;
                }
                // Marks the old file had for this file that this Nvim has
                // nowhere to put. They are owned here, so they are freed
                // whether or not writing them worked.
                let mut ret = kSDWriteSuccessful;
                for i in 0..(*file).additional_marks_size {
                    let entry = (*file).additional_marks.add(i);
                    if ret != kSDWriteFailed {
                        ret = shada_pack_entry(&raw mut self.packer, *entry, 0);
                    }
                    shada_free_shada_entry(entry);
                }
                xfree((*file).additional_marks.cast());
                if ret == kSDWriteFailed {
                    return kSDWriteFailed;
                }
            }
            kSDWriteSuccessful
        }
    }

    /// The merged histories, oldest entry first.
    unsafe fn pack_histories(&mut self) -> ShaDaWriteResult {
        unsafe {
            for i in 0..HIST_COUNT as usize {
                if !self.histories[i] {
                    continue;
                }
                hms_insert_whole_neovim_history(&raw mut (*self.wms).hms[i]);
                let mut cur = (*self.wms).hms[i].hmll.first;
                while !cur.is_null() {
                    if self.pack_freeing((*cur).data) == kSDWriteFailed {
                        return kSDWriteFailed;
                    }
                    cur = (*cur).next;
                }
            }
            kSDWriteSuccessful
        }
    }

    /// Write a slot-per-name array — the marks and the registers — whose
    /// empty slots are skipped.
    ///
    /// Takes the array by pointer: `&(*wms).field[..]` would be an autoref
    /// through a raw pointer, and the entries are freed as they go.
    unsafe fn pack_sparse<const N: usize>(
        &mut self,
        entries: *const [ShadaEntry; N],
    ) -> ShaDaWriteResult {
        unsafe {
            let entries = entries.cast::<ShadaEntry>();
            for i in 0..N {
                let entry = *entries.add(i);
                if entry.type_0 != kSDItemMissing && self.pack_freeing(entry) == kSDWriteFailed {
                    return kSDWriteFailed;
                }
            }
            kSDWriteSuccessful
        }
    }

    /// Write a filled-from-the-front array — the jumps and the changes —
    /// every entry of which is a real one.
    unsafe fn pack_dense(&mut self, entries: *const ShadaEntry, len: size_t) -> ShaDaWriteResult {
        unsafe {
            for i in 0..len {
                if self.pack_freeing(*entries.add(i)) == kSDWriteFailed {
                    return kSDWriteFailed;
                }
            }
            kSDWriteSuccessful
        }
    }

    /// Write one entry that `wms` owns, and release it.
    unsafe fn pack_freeing(&mut self, entry: ShadaEntry) -> ShaDaWriteResult {
        unsafe { shada_pack_pfreed_entry(&raw mut self.packer, entry, self.limits.max_kbyte) }
    }

    /// Write one entry that belongs to the caller.
    unsafe fn pack(&mut self, entry: ShadaEntry, max_kbyte: size_t) -> ShaDaWriteResult {
        unsafe { shada_pack_entry(&raw mut self.packer, entry, max_kbyte) }
    }

    /// Flush the output and release everything, however the write went.
    fn finish(&mut self) {
        // SAFETY: everything here was allocated by this write.
        unsafe {
            for i in 0..HIST_COUNT as usize {
                if self.histories[i] {
                    hms_dealloc(&raw mut (*self.wms).hms[i]);
                }
            }
            let marks = &raw mut (*self.wms).file_marks;
            for i in 0..(*marks).set.h.n_keys as usize {
                xfree((*(*marks).set.keys.add(i)).cast_mut().cast());
                xfree(*(*marks).values.add(i));
            }
            map_destroy_cstr_t_ptr_t(marks);
            set_destroy_ptr_t(&raw mut self.removable_bufs);
            self.packer.packer_flush.expect("shada: no flush")(&raw mut self.packer);
            // Only the names were borrowed; the variables themselves went
            // out as they were found.
            set_destroy_cstr_t(&raw mut (*self.wms).dumped_variables);
            xfree(self.wms.cast());
        }
    }
}

/// Whether a variable's value can be written at all.
///
/// Functions have no representation in the format, and a container that
/// refers to itself would not terminate the encoder.
unsafe fn writable_value(vartv: &typval_T) -> bool {
    unsafe {
        match vartv.v_type {
            VAR_FUNC | VAR_PARTIAL => false,
            VAR_DICT => {
                let di = vartv.vval.v_dict;
                let copy_id = get_copy_id();
                set_ref_in_ht(&raw mut (*di).dv_hashtab, copy_id, core::ptr::null_mut())
                    || copy_id != (*di).dv_copyID
            }
            VAR_LIST => {
                let l = vartv.vval.v_list;
                let copy_id = get_copy_id();
                set_ref_in_list_items(l, copy_id, core::ptr::null_mut())
                    || copy_id != (*l).lv_copyID
            }
            _ => true,
        }
    }
}
