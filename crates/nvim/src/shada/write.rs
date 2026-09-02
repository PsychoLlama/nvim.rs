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
        let mut max_kbyte_i = unsafe { get_shada_parameter('s' as c_int) };
        if max_kbyte_i < 0 {
            // Not given: the format's own default.
            max_kbyte_i = 10;
        }
        if max_kbyte_i == 0 {
            return None;
        }
        let mut max_reg_lines = unsafe { get_shada_parameter('<' as c_int) };
        if max_reg_lines < 0 {
            max_reg_lines = unsafe { get_shada_parameter('"' as c_int) };
        }
        Some(Limits {
            max_kbyte: max_kbyte_i as size_t,
            max_reg_lines,
            num_marked_files: unsafe { get_shada_parameter('\'' as c_int) } as size_t,
            global_vars: !unsafe { find_shada_parameter('!' as c_int) }.is_null(),
            global_marks: unsafe { get_shada_parameter('f' as c_int) } != 0,
        })
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
    let Some(limits) = (unsafe { Limits::from_shada_option() }) else {
        return kSDWriteSuccessful;
    };

    let wms = shada_heap(WriteMergerState::EMPTY);
    let histories = unsafe { init_histories(wms, !sd_reader.is_null()) };
    let srni_flags = wanted_kinds(&limits, &histories);

    let mut writing = Writing {
        wms,
        removable_bufs: SET_PTR_INIT,
        packer: unsafe { packer_buffer_for_file(sd_writer) },
        limits,
        histories,
    };

    // Recording where the cursor is in every window is what makes the
    // `"` mark right on exit. It also means `:wshada` moves that mark
    // to the cursor, as `:wviminfo` did.
    for wp in tab_windows() {
        unsafe { set_last_cursor(wp.raw()) };
    }
    unsafe { find_removable_bufs(&raw mut writing.removable_bufs) };

    let ret = unsafe { writing.run(sd_reader, srni_flags) };
    writing.finish();
    ret
}

/// Start a history merger for each history type `'shada'` keeps something
/// of, and answer which those are.
unsafe fn init_histories(wms: *mut WriteMergerState, merging: bool) -> [bool; HIST_COUNT as usize] {
    let mut wanted = [false; HIST_COUNT as usize];
    for (i, wanted) in wanted.iter_mut().enumerate() {
        let mut num_saved = unsafe { get_shada_parameter(hist_type2char(i as c_int)) };
        if num_saved == -1 {
            num_saved = p_hi.get() as c_int;
        }
        if num_saved > 0 {
            *wanted = true;
            let hms = unsafe { &raw mut (*wms).hms[i] };
            let num_saved = num_saved as size_t;
            unsafe { hms_init(hms, i as uint8_t, num_saved, merging, false) };
        }
    }
    wanted
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
        if unsafe { self.write_header() } == kSDWriteFailed {
            return kSDWriteFailed;
        }
        if !unsafe { find_shada_parameter('%' as c_int) }.is_null()
            && unsafe { self.write_buflist() } == kSDWriteFailed
        {
            return kSDWriteFailed;
        }
        // Variables go out as they are found rather than into `wms`;
        // only their names are kept, so that the merge knows which of
        // the old file's variables have already been written.
        if self.limits.global_vars && unsafe { self.dump_variables() } == kSDWriteFailed {
            return kSDWriteFailed;
        }

        unsafe { self.collect_jumps() };
        unsafe { self.collect_search_patterns() };
        unsafe { self.collect_global_marks() };
        if self.limits.registers() {
            unsafe { shada_initialize_registers(self.wms, self.limits.max_reg_lines) };
        }
        unsafe { self.collect_buffer_marks() };

        // Whatever the old file holds that this Nvim has no opinion
        // about, or a staler one.
        let mut ret = kSDWriteSuccessful;
        if !sd_reader.is_null() {
            let max_kbyte = self.limits.max_kbyte;
            let wms = self.wms;
            let packer = &raw mut self.packer;
            let srww_ret =
                unsafe { shada_read_when_writing(sd_reader, srni_flags, max_kbyte, wms, packer) };
            if srww_ret != kSDWriteSuccessful {
                ret = srww_ret;
            }
        }

        unsafe { self.update_numbered_marks() };
        if unsafe { self.pack_everything() } == kSDWriteFailed {
            return kSDWriteFailed;
        }
        ret
    }

    /// What this Nvim was. Nothing ever reads it back; it is there for
    /// anyone looking at the file by hand.
    unsafe fn write_header(&mut self) -> ShaDaWriteResult {
        let mut header = DictBuf::<5>::new();
        header
            .insert(c"generator", Object::string(static_cstring(c"nvim")))
            .insert(c"version", Object::string(static_cstring(LONG_VERSION)))
            .insert(
                c"max_kbyte",
                Object::integer(self.limits.max_kbyte as Integer),
            )
            .insert(c"pid", Object::integer(os_get_pid()))
            .insert(
                c"encoding",
                Object::string(unsafe { cstr_as_string(p_enc.get()) }),
            );
        let entry = ShadaEntry {
            can_free_entry: false,
            timestamp: os_time(),
            data: ShadaEntryData::Header(header.dict()),
            additional_data: core::ptr::null_mut(),
        };
        unsafe { self.pack(entry, 0) }
    }

    /// The list of files this Nvim has buffers for, so that a later start
    /// can reopen them.
    unsafe fn write_buflist(&mut self) -> ShaDaWriteResult {
        let entry = unsafe { shada_get_buflist(&raw mut self.removable_bufs) };
        let ret = unsafe { self.pack(entry, 0) };
        unsafe { xfree(entry.data.buffer_list().buffers.cast()) };
        ret
    }

    /// Every global variable `'shada'` says to keep, written as it is found.
    ///
    /// A container that turns out to be part of a cycle is skipped: the
    /// encoder would not terminate on it.
    unsafe fn dump_variables(&mut self) -> ShaDaWriteResult {
        let mut var_iter: Option<usize> = None;
        let timestamp = os_time();
        loop {
            let mut vartv: typval_T = unsafe { core::mem::zeroed() };
            let mut name: *const c_char = core::ptr::null();
            var_iter = unsafe {
                var_shada_iter(var_iter, &raw mut name, &raw mut vartv, VAR_FLAVOUR_SHADA)
            };
            if name.is_null() {
                return kSDWriteSuccessful;
            }

            if !unsafe { writable_value(&vartv) } {
                unsafe { tv_clear(&raw mut vartv) };
                if var_iter.is_none() {
                    return kSDWriteSuccessful;
                }
                continue;
            }

            // The entry takes a copy, which the pack frees; the value
            // the iterator handed over is this function's to release.
            let mut tgttv: typval_T = unsafe { core::mem::zeroed() };
            unsafe { tv_copy(&raw mut vartv, &raw mut tgttv) };
            let entry = ShadaEntry {
                can_free_entry: false,
                timestamp,
                data: ShadaEntryData::Variable(global_var {
                    name: name.cast_mut(),
                    value: tgttv,
                }),
                additional_data: core::ptr::null_mut(),
            };
            let ret = unsafe { self.pack(entry, self.limits.max_kbyte) };
            unsafe { tv_clear(&raw mut vartv) };
            unsafe { tv_clear(&raw mut tgttv) };
            if ret == kSDWriteFailed {
                return kSDWriteFailed;
            }
            if ret == kSDWriteSuccessful {
                unsafe {
                    set_put_cstr_t(
                        &raw mut (*self.wms).dumped_variables,
                        name,
                        core::ptr::null_mut(),
                    )
                };
            }
            if var_iter.is_none() {
                return kSDWriteSuccessful;
            }
        }
    }

    /// The jump list, as far back as `'shada'` keeps files' marks.
    unsafe fn collect_jumps(&mut self) {
        if self.limits.num_marked_files > 0 {
            unsafe {
                (*self.wms).jumps_size = shada_init_jumps(
                    (&raw mut (*self.wms).jumps).cast::<ShadaEntry>(),
                    &raw mut self.removable_bufs,
                )
            };
        }
    }

    /// The last search and substitute patterns, and the last `:substitute`
    /// replacement string. All three ride on the search history's setting.
    unsafe fn collect_search_patterns(&mut self) {
        if !self.histories[HIST_SEARCH as usize] {
            return;
        }
        let highlighted =
            !(no_hlsearch.get() || !unsafe { find_shada_parameter('h' as c_int) }.is_null());
        let last_used = search_was_last_used();

        let slot = unsafe { &raw mut (*self.wms).search_pattern };
        unsafe {
            add_search_pattern(
                slot,
                Some(get_search_pattern),
                false,
                last_used,
                highlighted,
            )
        };
        let slot = unsafe { &raw mut (*self.wms).sub_search_pattern };
        unsafe {
            add_search_pattern(
                slot,
                Some(get_substitute_pattern),
                true,
                last_used,
                highlighted,
            )
        };

        let mut sub: SubReplacementString = unsafe { core::mem::zeroed() };
        unsafe { sub_get_replacement(&raw mut sub) };
        // An empty replacement string is not worth storing.
        if !sub.sub.is_null() {
            let entry = ShadaEntry {
                can_free_entry: false,
                timestamp: sub.timestamp,
                data: ShadaEntryData::SubString(sub_string { sub: sub.sub }),
                additional_data: sub.additional_data,
            };
            unsafe { (*self.wms).replacement = entry };
        }
    }

    /// The lettered and numbered global marks.
    ///
    /// A mark on a file no buffer holds still names the file; one on a
    /// buffer that has gone, or that sits on a removable medium, is dropped.
    unsafe fn collect_global_marks(&mut self) {
        if !self.limits.global_marks {
            return;
        }
        let mut mark_iter: *const c_void = core::ptr::null();
        let mut digit_mark_idx = 0;
        loop {
            let mut name: c_char = NUL as c_char;
            let mut fm: xfmark_T = unsafe { core::mem::zeroed() };
            mark_iter = unsafe { mark_global_iter(mark_iter, &raw mut name, &raw mut fm) };
            if name as c_int == NUL {
                return;
            }

            if let Some(fname) = unsafe { self.mark_fname(&fm) } {
                let entry = ShadaEntry {
                    can_free_entry: false,
                    timestamp: fm.fmark.timestamp,
                    data: ShadaEntryData::GlobalMark(shada_filemark {
                        name,
                        mark: fm.fmark.mark,
                        fname: fname.cast_mut(),
                    }),
                    additional_data: fm.fmark.additional_data,
                };
                // The ten numbered marks are simply the ten most
                // recent; their names are assigned on the way out.
                if ascii_isdigit(name as c_int) {
                    unsafe { replace_numbered_mark(self.wms, digit_mark_idx, entry) };
                    digit_mark_idx += 1;
                } else {
                    unsafe { (*self.wms).global_marks[mark_global_index(name) as usize] = entry };
                }
            }

            if mark_iter.is_null() {
                return;
            }
        }
    }

    /// The file name to record a global mark against, or `None` when the
    /// mark is not worth keeping.
    unsafe fn mark_fname(&mut self, fm: &xfmark_T) -> Option<*const c_char> {
        if fm.fmark.fnum == 0 {
            debug_assert!(!fm.fname.is_null(), "shada: a mark with no buffer or file");
            return (!unsafe { shada_removable(fm.fname) }).then_some(fm.fname);
        }
        let buf = find_buf(fm.fmark.fnum).map_or(core::ptr::null_mut(), |mut b| b.raw());
        if buf.is_null()
            || unsafe { (*buf).b_ffname.is_null() }
            || unsafe { set_has_ptr_t(&raw mut self.removable_bufs, buf.cast::<c_void>()) }
        {
            return None;
        }
        Some(unsafe { (*buf).b_ffname })
    }

    /// Every buffer's local marks and change list, keyed by file name so
    /// that the merge can find the same file's marks in the old file.
    unsafe fn collect_buffer_marks(&mut self) {
        if self.limits.num_marked_files == 0 {
            return;
        }
        for buf in buffers() {
            if !unsafe { ignore_buf(buf.raw(), &raw mut self.removable_bufs) } {
                unsafe { self.collect_one_buffer(buf.raw()) };
            }
        }
    }

    unsafe fn collect_one_buffer(&mut self, buf: *mut buf_T) {
        let fname = unsafe { (*buf).b_ffname };
        let filemarks = unsafe { self.file_marks_for(fname) };

        let mut mark_iter: *const c_void = core::ptr::null();
        loop {
            let mut fm: fmark_T = unsafe { core::mem::zeroed() };
            let mut name: c_char = NUL as c_char;
            mark_iter = unsafe { mark_buffer_iter(mark_iter, buf, &raw mut name, &raw mut fm) };
            if name as c_int == NUL {
                break;
            }
            let entry = ShadaEntry {
                can_free_entry: false,
                timestamp: fm.timestamp,
                data: ShadaEntryData::LocalMark(shada_filemark {
                    name,
                    mark: fm.mark,
                    fname,
                }),
                additional_data: fm.additional_data,
            };
            unsafe { (*filemarks).marks[mark_local_index(name) as usize] = entry };
            unsafe {
                (*filemarks).greatest_timestamp = (*filemarks).greatest_timestamp.max(fm.timestamp)
            };
            if mark_iter.is_null() {
                break;
            }
        }

        for i in 0..unsafe { (*buf).b_changelistlen } as usize {
            let fm = unsafe { (*buf).b_changelist[i].clone() };
            let entry = ShadaEntry {
                can_free_entry: false,
                timestamp: fm.timestamp,
                data: ShadaEntryData::Change(shada_filemark {
                    name: 0,
                    mark: fm.mark,
                    fname,
                }),
                additional_data: fm.additional_data,
            };
            unsafe { (*filemarks).changes[i] = entry };
            unsafe {
                (*filemarks).greatest_timestamp = (*filemarks).greatest_timestamp.max(fm.timestamp)
            };
        }
        unsafe { (*filemarks).changes_size = (*buf).b_changelistlen as size_t };
    }

    /// The slot one file's marks are collected into, made on first use.
    /// The map owns both its keys and its values.
    unsafe fn file_marks_for(&mut self, fname: *const c_char) -> *mut FileMarks {
        let mut key_alloc: *mut cstr_t = core::ptr::null_mut();
        let mut new_item = false;
        let slot = unsafe {
            map_put_ref_cstr_t_ptr_t(
                &raw mut (*self.wms).file_marks,
                fname,
                &raw mut key_alloc,
                &raw mut new_item,
            )
        };
        if new_item {
            unsafe { *key_alloc = xstrdup(fname) };
        }
        if unsafe { (*slot).is_null() } {
            let marks = shada_heap(FileMarks::EMPTY).cast::<c_void>();
            unsafe { *slot = marks };
        }
        unsafe { (*slot).cast::<FileMarks>() }
    }

    /// Put the cursor's position in at `'0`, shifting the other numbered
    /// marks down and dropping `'9`.
    unsafe fn update_numbered_marks(&mut self) {
        if !self.limits.global_marks
            || unsafe { ignore_buf(curbuf.get(), &raw mut self.removable_bufs) }
            || unsafe { (*curwin.get()).w_cursor.lnum } == 0
        {
            return;
        }
        let entry = ShadaEntry {
            can_free_entry: false,
            timestamp: os_time(),
            data: ShadaEntryData::GlobalMark(shada_filemark {
                name: '0' as c_char,
                mark: unsafe { (*curwin.get()).w_cursor },
                fname: unsafe { (*curbuf.get()).b_ffname },
            }),
            additional_data: core::ptr::null_mut(),
        };
        unsafe { replace_numbered_mark(self.wms, 0, entry) };
    }

    /// Write everything the merge left in `wms`, in the order the format
    /// wants it.
    unsafe fn pack_everything(&mut self) -> ShaDaWriteResult {
        let wms = self.wms;
        if unsafe { self.pack_sparse(&raw const (*wms).global_marks) } == kSDWriteFailed
            || unsafe { self.pack_sparse(&raw const (*wms).numbered_marks) } == kSDWriteFailed
            || unsafe { self.pack_sparse(&raw const (*wms).registers) } == kSDWriteFailed
            || unsafe { self.pack_dense((&raw const (*wms).jumps).cast(), (*wms).jumps_size) }
                == kSDWriteFailed
        {
            return kSDWriteFailed;
        }
        for entry in [
            unsafe { (*wms).search_pattern },
            unsafe { (*wms).sub_search_pattern },
            unsafe { (*wms).replacement },
        ] {
            if !entry.data.is_missing() && unsafe { self.pack_freeing(entry) } == kSDWriteFailed {
                return kSDWriteFailed;
            }
        }
        if unsafe { self.pack_file_marks() } == kSDWriteFailed {
            return kSDWriteFailed;
        }
        unsafe { self.pack_histories() }
    }

    /// Every file's marks, most recently touched file first, as many files
    /// as `'shada'` keeps.
    unsafe fn pack_file_marks(&mut self) -> ShaDaWriteResult {
        let mut all: Vec<*mut FileMarks> = Vec::new();
        let marks = unsafe { &(*self.wms).file_marks };
        for i in 0..marks.set.h.n_keys as usize {
            all.push(unsafe { (*marks.values.add(i)).cast::<FileMarks>() });
        }
        unsafe {
            qsort(
                all.as_mut_ptr().cast(),
                all.len(),
                size_of::<*mut FileMarks>(),
                Some(compare_file_marks),
            )
        };

        let to_dump = all.len().min(self.limits.num_marked_files);
        for &file in &all[..to_dump] {
            if unsafe { self.pack_sparse(&raw const (*file).marks) } == kSDWriteFailed
                || unsafe {
                    self.pack_dense((&raw const (*file).changes).cast(), (*file).changes_size)
                } == kSDWriteFailed
            {
                return kSDWriteFailed;
            }
            // Marks the old file had for this file that this Nvim has
            // nowhere to put. They are owned here, so they are freed
            // whether or not writing them worked.
            let mut ret = kSDWriteSuccessful;
            for i in 0..unsafe { (*file).additional_marks_size } {
                let entry = unsafe { (*file).additional_marks.add(i) };
                if ret != kSDWriteFailed {
                    ret = unsafe { shada_pack_entry(&raw mut self.packer, *entry, 0) };
                }
                unsafe { shada_free_shada_entry(entry) };
            }
            unsafe { xfree((*file).additional_marks.cast()) };
            if ret == kSDWriteFailed {
                return kSDWriteFailed;
            }
        }
        kSDWriteSuccessful
    }

    /// The merged histories, oldest entry first.
    unsafe fn pack_histories(&mut self) -> ShaDaWriteResult {
        for i in 0..HIST_COUNT as usize {
            if !self.histories[i] {
                continue;
            }
            unsafe { hms_insert_whole_neovim_history(&raw mut (*self.wms).hms[i]) };
            let mut cur = unsafe { (*self.wms).hms[i].hmll.first };
            while !cur.is_null() {
                if unsafe { self.pack_freeing((*cur).data) } == kSDWriteFailed {
                    return kSDWriteFailed;
                }
                cur = unsafe { (*cur).next };
            }
        }
        kSDWriteSuccessful
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
        let entries = entries.cast::<ShadaEntry>();
        for i in 0..N {
            let entry = unsafe { *entries.add(i) };
            if !entry.data.is_missing() && unsafe { self.pack_freeing(entry) } == kSDWriteFailed {
                return kSDWriteFailed;
            }
        }
        kSDWriteSuccessful
    }

    /// Write a filled-from-the-front array — the jumps and the changes —
    /// every entry of which is a real one.
    unsafe fn pack_dense(&mut self, entries: *const ShadaEntry, len: size_t) -> ShaDaWriteResult {
        for i in 0..len {
            if unsafe { self.pack_freeing(*entries.add(i)) } == kSDWriteFailed {
                return kSDWriteFailed;
            }
        }
        kSDWriteSuccessful
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
        for i in 0..HIST_COUNT as usize {
            if self.histories[i] {
                unsafe { hms_dealloc(&raw mut (*self.wms).hms[i]) };
            }
        }
        let marks = unsafe { &raw mut (*self.wms).file_marks };
        for i in 0..unsafe { (*marks).set.h.n_keys } as usize {
            unsafe { xfree((*(*marks).set.keys.add(i)).cast_mut().cast()) };
            unsafe { xfree(*(*marks).values.add(i)) };
        }
        unsafe { map_destroy_cstr_t_ptr_t(marks) };
        unsafe { set_destroy_ptr_t(&raw mut self.removable_bufs) };
        unsafe { self.packer.packer_flush.expect("shada: no flush")(&raw mut self.packer) };
        // Only the names were borrowed; the variables themselves went
        // out as they were found.
        unsafe { set_destroy_cstr_t(&raw mut (*self.wms).dumped_variables) };
        unsafe { xfree(self.wms.cast()) };
    }
}

/// Whether a variable's value can be written at all.
///
/// Functions have no representation in the format, and a container that
/// refers to itself would not terminate the encoder.
unsafe fn writable_value(vartv: &typval_T) -> bool {
    match vartv.v_type {
        VAR_FUNC | VAR_PARTIAL => false,
        VAR_DICT => {
            let di = unsafe { vartv.vval.v_dict };
            let copy_id = unsafe { get_copy_id() };
            unsafe {
                set_ref_in_ht(&raw mut (*di).dv_hashtab, copy_id, core::ptr::null_mut())
                    || copy_id != (*di).dv_copyID
            }
        }
        VAR_LIST => {
            let l = unsafe { vartv.vval.v_list };
            let copy_id = unsafe { get_copy_id() };
            unsafe {
                set_ref_in_list_items(l, copy_id, core::ptr::null_mut())
                    || copy_id != (*l).lv_copyID
            }
        }
        _ => true,
    }
}
