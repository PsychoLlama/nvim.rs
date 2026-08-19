//! Applying a ShaDa file to the running editor.
//!
//! `shada_read` walks the entries an already-opened file yields and puts each
//! one where it belongs — registers, marks, histories, global variables, the
//! buffer list — subject to the `kSDRead*` flags that say which kinds the
//! caller asked for and to the `'shada'` option's limits.
//!
//! Almost every entry the file yields is *given away* rather than copied:
//! the strings it was read into become the register's, the mark's or the
//! history's. That is why each applier ends either by handing the entry on
//! or by freeing it, and never both.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint};

use super::*;
use crate::types::{VAR_UNKNOWN, Vv, kListLenUnknown};

/// What a mark restored from a file starts its view at: nothing is known
/// about where the window was scrolled to.
const INIT_FMARKV: fmarkv_T = fmarkv_T {
    topline_offset: MAXLNUM as linenr_T,
    skipcol: 0,
};

/// The kinds of entry a read with these flags is looking for.
///
/// The caller's flags and the `'shada'` option between them decide this;
/// when they agree on nothing there is no point reading the file at all.
unsafe fn wanted_kinds(flags: c_int, want_marks: bool, get_old_files: bool) -> c_uint {
    unsafe {
        let mut kinds: c_uint = 0;
        if flags & kShaDaWantInfo as c_int != 0 {
            kinds |= kSDReadUndisableableData | kSDReadRegisters | kSDReadGlobalMarks;
            if p_hi.get() != 0 {
                kinds |= kSDReadHistory;
            }
            if !find_shada_parameter('!' as c_int).is_null() {
                kinds |= kSDReadVariables;
            }
            // The buffer list is only restored into an Nvim that was not
            // given files to edit.
            if !find_shada_parameter('%' as c_int).is_null()
                && (*(*curwin.get()).w_alist).al_ga.ga_len == 0
            {
                kinds |= kSDReadBufferList;
            }
        }
        if want_marks && get_shada_parameter('\'' as c_int) > 0 {
            kinds |= kSDReadLocalMarks | kSDReadChanges;
        }
        // File marks are also where `v:oldfiles` comes from.
        if get_old_files {
            kinds |= kSDReadLocalMarks;
        }
        kinds
    }
}

/// What one pass of [`shada_read`] carries between entries.
struct Reading {
    /// `:rshada!` — take what the file says whatever this session holds.
    force: bool,
    /// Whether marks are being restored, as opposed to only walked over for
    /// the file names they carry.
    want_marks: bool,
    /// Whether `v:oldfiles` is being built from this file.
    get_old_files: bool,
    /// The list behind `v:oldfiles`.
    oldfiles_list: *mut list_T,
    /// The file names already in `oldfiles_list`.
    oldfiles_set: Set_cstr_t,
    /// Buffers whose change list grew; the windows showing them are moved
    /// to the end of it once the whole file has been read.
    cl_bufs: Set_ptr_t,
    /// File name to the loaded buffer for it, or null when there is none.
    /// Memoises the walk of the buffer list; the keys are owned copies.
    fname_bufs: Map_cstr_t_ptr_t,
    /// One merger per history type, used only when histories are wanted.
    hms: [HistoryMergerState; HIST_COUNT as usize],
}

/// Read a ShaDa file and apply it.
pub(crate) unsafe fn shada_read(sd_reader: *mut FileDescriptor, flags: c_int) {
    unsafe {
        let force = flags & kShaDaForceit as c_int != 0;
        let mut oldfiles_list = get_vim_var_list(Vv::Oldfiles);
        // `v:oldfiles` is only filled in while it is still empty, so that a
        // second file does not append to the first one's answer.
        let get_old_files = flags & (kShaDaGetOldfiles | kShaDaForceit) as c_int != 0
            && (force || tv_list_len(oldfiles_list) == 0);
        let want_marks = flags & kShaDaWantMarks as c_int != 0;

        let srni_flags = wanted_kinds(flags, want_marks, get_old_files);
        if srni_flags == 0 {
            return;
        }

        let mut hms: [HistoryMergerState; HIST_COUNT as usize] = core::mem::zeroed();
        if srni_flags & kSDReadHistory != 0 {
            for (i, hms) in hms.iter_mut().enumerate() {
                hms_init(hms, i as uint8_t, p_hi.get() as size_t, true, true);
            }
        }
        if get_old_files && (oldfiles_list.is_null() || force) {
            oldfiles_list = tv_list_alloc(kListLenUnknown as ptrdiff_t);
            set_vim_var_list(Vv::Oldfiles, oldfiles_list);
        }

        let mut state = Reading {
            force,
            want_marks,
            get_old_files,
            oldfiles_list,
            oldfiles_set: Set_cstr_t {
                h: MAPHASH_INIT,
                keys: core::ptr::null_mut(),
            },
            cl_bufs: Set_ptr_t {
                h: MAPHASH_INIT,
                keys: core::ptr::null_mut(),
            },
            fname_bufs: MAP_INIT,
            hms,
        };

        let mut entry: ShadaEntry = core::mem::zeroed();
        loop {
            match shada_read_next_item(sd_reader, &raw mut entry, srni_flags, 0) {
                kSDReadStatusSuccess => {}
                kSDReadStatusFinished => break,
                // One bad entry is skipped; a file that turned out not to be
                // ShaDa, or that stopped reading, ends the pass with what has
                // been applied so far left in place.
                kSDReadStatusMalformed => continue,
                _ => break,
            }
            state.apply(entry);
        }

        state.finish(srni_flags);
    }
}

impl Reading {
    /// Put one entry from the file where it belongs.
    unsafe fn apply(&mut self, mut entry: ShadaEntry) {
        unsafe {
            match entry.type_0 {
                kSDItemMissing => unreachable!("shada: read an entry with no type"),
                // Only reached with `kSDReadUnknown`, which a plain read
                // never asks for.
                kSDItemUnknown => {}
                kSDItemHeader => shada_free_shada_entry(&raw mut entry),
                kSDItemSearchPattern => self.apply_search_pattern(entry),
                kSDItemSubString => self.apply_sub_string(entry),
                kSDItemHistoryEntry => self.apply_history(entry),
                kSDItemRegister => self.apply_register(entry),
                kSDItemVariable => apply_variable(entry),
                kSDItemGlobalMark | kSDItemJump => self.apply_file_mark(entry),
                kSDItemBufferList => apply_buffer_list(entry),
                kSDItemLocalMark | kSDItemChange => self.apply_buffer_mark(entry),
                _ => {}
            }
        }
    }

    /// A search or substitute pattern. The one this session already has
    /// wins a tie, unless the read was forced.
    unsafe fn apply_search_pattern(&self, mut entry: ShadaEntry) {
        unsafe {
            let pat = entry.data.search_pattern;
            let is_sub = pat.is_substitute_pattern;
            if !self.force {
                let mut current: SearchPattern = core::mem::zeroed();
                if is_sub {
                    get_substitute_pattern(&raw mut current);
                } else {
                    get_search_pattern(&raw mut current);
                }
                if !current.pat.is_null() && current.timestamp >= entry.timestamp {
                    shada_free_shada_entry(&raw mut entry);
                    return;
                }
            }

            // The pattern takes the entry's string and extra data over.
            let spat = SearchPattern {
                pat: pat.pat.data,
                patlen: pat.pat.size,
                magic: pat.magic,
                no_scs: !pat.smartcase,
                timestamp: entry.timestamp,
                off: SearchOffset {
                    dir: if pat.search_backward { b'?' } else { b'/' } as c_char,
                    line: pat.has_line_offset,
                    end: pat.place_cursor_at_end,
                    off: pat.offset as int64_t,
                },
                additional_data: entry.additional_data,
            };
            if is_sub {
                set_substitute_pattern(spat);
            } else {
                set_search_pattern(spat);
            }
            if pat.is_last_used {
                set_last_used_pattern(is_sub);
                set_no_hlsearch(!pat.highlighted);
            }
        }
    }

    /// The last `:substitute` replacement string.
    unsafe fn apply_sub_string(&self, mut entry: ShadaEntry) {
        unsafe {
            if !self.force {
                let mut current: SubReplacementString = core::mem::zeroed();
                sub_get_replacement(&raw mut current);
                if !current.sub.is_null() && current.timestamp >= entry.timestamp {
                    shada_free_shada_entry(&raw mut entry);
                    return;
                }
            }
            let sub = entry.data.sub_string.sub;
            sub_set_replacement(SubReplacementString {
                sub,
                timestamp: entry.timestamp,
                additional_data: entry.additional_data,
            });
            // Without a `regtilde` call the restored string is close to
            // useless: `s//~` does not reach it until one has happened. Vim
            // did not do this.
            regtilde(sub, magic_isset() as c_int, false);
        }
    }

    /// One history entry, handed to the merger for its type.
    unsafe fn apply_history(&mut self, mut entry: ShadaEntry) {
        unsafe {
            let histtype = entry.data.history_item.histtype as c_uint;
            if histtype >= HIST_COUNT {
                shada_free_shada_entry(&raw mut entry);
                return;
            }
            hms_insert(&raw mut self.hms[histtype as usize], entry, true);
        }
    }

    /// One register. The register this session already holds wins a tie,
    /// unless the read was forced.
    unsafe fn apply_register(&self, mut entry: ShadaEntry) {
        unsafe {
            let reg = entry.data.reg;
            if reg.type_0 != kMTCharWise && reg.type_0 != kMTLineWise && reg.type_0 != kMTBlockWise
            {
                shada_free_shada_entry(&raw mut entry);
                return;
            }
            if !self.force {
                let current = op_reg_get(reg.name);
                if current.is_null() || (*current).timestamp >= entry.timestamp {
                    shada_free_shada_entry(&raw mut entry);
                    return;
                }
            }
            let stored = op_reg_set(
                reg.name,
                yankreg_T {
                    y_array: reg.contents,
                    y_size: reg.contents_size,
                    y_type: reg.type_0,
                    y_width: reg.width as colnr_T,
                    timestamp: entry.timestamp,
                    additional_data: entry.additional_data,
                },
                reg.is_unnamed,
            );
            if !stored {
                shada_free_shada_entry(&raw mut entry);
            }
        }
    }

    /// A global mark or a jump-list entry. Both name a *file*, so a loaded
    /// buffer for that name is looked for first: when there is one the mark
    /// refers to it by number and the file name is dropped.
    unsafe fn apply_file_mark(&mut self, mut entry: ShadaEntry) {
        unsafe {
            let buf = buffer_for_fname(&raw mut self.fname_bufs, entry.data.filemark.fname);
            if !buf.is_null() {
                xfree(entry.data.filemark.fname.cast());
                entry.data.filemark.fname = core::ptr::null_mut();
            }
            let fm = xfmark_T {
                fmark: fmark_T {
                    mark: entry.data.filemark.mark,
                    fnum: if buf.is_null() {
                        0
                    } else {
                        (*buf).handle as c_int
                    },
                    timestamp: entry.timestamp,
                    view: INIT_FMARKV,
                    additional_data: entry.additional_data,
                },
                // Null exactly when a buffer was found, from just above.
                fname: entry.data.filemark.fname,
            };
            if entry.type_0 == kSDItemGlobalMark {
                if !mark_set_global(entry.data.filemark.name, fm, !self.force) {
                    shada_free_shada_entry(&raw mut entry);
                }
                return;
            }
            insert_jump(fm, buf, entry);
        }
    }

    /// A buffer-local mark or change-list entry.
    ///
    /// These are also what `v:oldfiles` is built from, which is why the file
    /// name matters even when the marks themselves were not asked for.
    unsafe fn apply_buffer_mark(&mut self, mut entry: ShadaEntry) {
        unsafe {
            if self.get_old_files
                && !set_has_cstr_t(&raw mut self.oldfiles_set, entry.data.filemark.fname)
            {
                // The entry's own string can be handed to the list, unless
                // the mark below is still going to need it.
                let fname = if self.want_marks {
                    xstrdup(entry.data.filemark.fname)
                } else {
                    entry.data.filemark.fname
                };
                set_put_cstr_t(&raw mut self.oldfiles_set, fname, core::ptr::null_mut());
                tv_list_append_allocated_string(self.oldfiles_list, fname);
                if !self.want_marks {
                    entry.data.filemark.fname = core::ptr::null_mut();
                }
            }
            if !self.want_marks {
                shada_free_shada_entry(&raw mut entry);
                return;
            }

            // A mark on a file no buffer is holding has nowhere to go.
            let buf = buffer_for_fname(&raw mut self.fname_bufs, entry.data.filemark.fname);
            if buf.is_null() {
                shada_free_shada_entry(&raw mut entry);
                return;
            }
            let fm = fmark_T {
                mark: entry.data.filemark.mark,
                fnum: (*buf).handle as c_int,
                timestamp: entry.timestamp,
                view: INIT_FMARKV,
                additional_data: entry.additional_data,
            };
            if entry.type_0 == kSDItemLocalMark {
                if !mark_set_local(entry.data.filemark.name, buf, fm, !self.force) {
                    shada_free_shada_entry(&raw mut entry);
                    return;
                }
            } else {
                set_put_ptr_t(&raw mut self.cl_bufs, buf.cast(), core::ptr::null_mut());
                insert_change(buf, fm);
            }
            // The mark took the extra data; only the file name is left.
            xfree(entry.data.filemark.fname.cast());
        }
    }

    /// Everything that could only be done once the whole file had been read.
    fn finish(&mut self, srni_flags: c_uint) {
        // SAFETY: the editor's own state, on the main thread.
        unsafe {
            // The mergers hold both the file's history and Nvim's; folding
            // in what is left of Nvim's makes the merged ring the history.
            if srni_flags & kSDReadHistory != 0 {
                for hms in &mut self.hms {
                    hms_insert_whole_neovim_history(hms);
                    hms_to_history(hms);
                    hms_dealloc(hms);
                }
            }
            // A window showing a buffer whose change list grew sits at the
            // end of it, as if the changes had just been made.
            if self.cl_bufs.h.n_occupied != 0 {
                for wp in all_windows() {
                    if set_has_ptr_t(&raw mut self.cl_bufs, (*wp).w_buffer.cast()) {
                        (*wp).w_changelistidx = (*(*wp).w_buffer).b_changelistlen;
                    }
                }
            }
            set_destroy_ptr_t(&raw mut self.cl_bufs);
            // The memo table owns its keys; the buffers it points at do not
            // belong to it.
            for i in 0..self.fname_bufs.set.h.n_keys as usize {
                xfree((*self.fname_bufs.set.keys.add(i)).cast_mut().cast());
            }
            map_destroy_cstr_t_ptr_t(&raw mut self.fname_bufs);
            // The names in this one were given to `v:oldfiles`.
            set_destroy_cstr_t(&raw mut self.oldfiles_set);
        }
    }
}

/// A global variable. `var_set_global` takes the value over, so the entry
/// is emptied of it before the rest is freed.
unsafe fn apply_variable(mut entry: ShadaEntry) {
    unsafe {
        var_set_global(entry.data.global_var.name, entry.data.global_var.value);
        entry.data.global_var.value.v_type = VAR_UNKNOWN;
        shada_free_shada_entry(&raw mut entry);
    }
}

/// The buffer list the file was written with: each name becomes a listed
/// buffer with its cursor where it was left.
unsafe fn apply_buffer_list(mut entry: ShadaEntry) {
    unsafe {
        let list = entry.data.buffer_list;
        for i in 0..list.size {
            let item = list.buffers.add(i);
            let sfname = path_try_shorten_fname((*item).fname);
            let buf = buflist_new((*item).fname, sfname, 0, BLN_LISTED as c_int);
            if buf.is_null() {
                continue;
            }
            free_fmark((*buf).b_last_cursor);
            (*buf).b_last_cursor = fmark_T {
                mark: (*item).pos,
                fnum: 0,
                timestamp: os_time(),
                view: INIT_FMARKV,
                additional_data: core::ptr::null_mut(),
            };
            let (lnum, col) = (
                (*buf).b_last_cursor.mark.lnum,
                (*buf).b_last_cursor.mark.col,
            );
            buflist_setfpos(buf, curwin.get(), lnum, col, false);
            xfree((*buf).additional_data.cast());
            (*buf).additional_data = (*item).additional_data;
            (*item).additional_data = core::ptr::null_mut();
        }
        shada_free_shada_entry(&raw mut entry);
    }
}

/// The loaded buffer editing `fname`, or null when there is none.
///
/// Answers are memoised in `fname_bufs`, whose keys are copies this makes
/// and the caller frees.
unsafe fn buffer_for_fname(fname_bufs: *mut Map_cstr_t_ptr_t, fname: *const c_char) -> *mut buf_T {
    unsafe {
        let mut key_alloc: *mut cstr_t = core::ptr::null_mut();
        let mut new_item = false;
        let slot =
            map_put_ref_cstr_t_ptr_t(fname_bufs, fname, &raw mut key_alloc, &raw mut new_item)
                .cast::<*mut buf_T>();
        if !new_item {
            return *slot;
        }
        *key_alloc = xstrdup(fname);

        let mut buf = firstbuf.get();
        while !buf.is_null() {
            if !(*buf).b_ffname.is_null() && path_fnamecmp(fname, (*buf).b_ffname) == 0 {
                *slot = buf;
                return buf;
            }
            buf = (*buf).b_next;
        }
        *slot = core::ptr::null_mut();
        core::ptr::null_mut()
    }
}

/// Put a jump into `curwin`'s jump list, which is kept oldest first.
///
/// A jump the list already holds — same position, same file — is dropped
/// rather than inserted twice, and so is one older than a list that is
/// already full.
unsafe fn insert_jump(fm: xfmark_T, buf: *mut buf_T, mut entry: ShadaEntry) {
    unsafe {
        let win = curwin.get();
        let mut i = (*win).w_jumplistlen;
        while i > 0 {
            let existing = (*win).w_jumplist[i as usize - 1];
            if existing.fmark.timestamp <= fm.fmark.timestamp {
                let same_file = if buf.is_null() {
                    !existing.fname.is_null() && strcmp(fm.fname, existing.fname) == 0
                } else {
                    fm.fmark.fnum == existing.fmark.fnum
                };
                if marks_equal(existing.fmark.mark, fm.fmark.mark) && same_file {
                    i = -1;
                }
                break;
            }
            i -= 1;
        }
        if i > 0 && (*win).w_jumplistlen == JUMPLISTSIZE {
            free_xfmark((*win).w_jumplist[0]);
        }
        let len = (*win).w_jumplistlen;
        let i = marklist_insert(&mut (*win).w_jumplist, len, i);
        if i == -1 {
            shada_free_shada_entry(&raw mut entry);
            return;
        }
        (*win).w_jumplist[i as usize] = fm;
        if (*win).w_jumplistlen < JUMPLISTSIZE {
            (*win).w_jumplistlen += 1;
        }
        // Keep the cursor into the list pointing at the same jump.
        if (*win).w_jumplistidx >= i && (*win).w_jumplistidx < (*win).w_jumplistlen {
            (*win).w_jumplistidx += 1;
        }
    }
}

/// [`insert_jump`] for a buffer's change list, which needs no file name to
/// compare on because every entry in it is in this buffer.
unsafe fn insert_change(buf: *mut buf_T, fm: fmark_T) {
    unsafe {
        let mut i = (*buf).b_changelistlen;
        while i > 0 {
            let existing = (*buf).b_changelist[i as usize - 1];
            if existing.timestamp <= fm.timestamp {
                if marks_equal(existing.mark, fm.mark) {
                    i = -1;
                }
                break;
            }
            i -= 1;
        }
        if i > 0 && (*buf).b_changelistlen == JUMPLISTSIZE {
            free_fmark((*buf).b_changelist[0]);
        }
        let len = (*buf).b_changelistlen;
        let i = marklist_insert(&mut (*buf).b_changelist, len, i);
        if i == -1 {
            xfree(fm.additional_data.cast());
            return;
        }
        (*buf).b_changelist[i as usize] = fm;
        if (*buf).b_changelistlen < JUMPLISTSIZE {
            (*buf).b_changelistlen += 1;
        }
    }
}

/// Release what an entry read from a file holds.
///
/// Entries built from Nvim's own state borrow their strings from it and say
/// so with `can_free_entry`; those are left alone.
pub(crate) unsafe fn shada_free_shada_entry(entry: *mut ShadaEntry) {
    unsafe {
        if entry.is_null() || !(*entry).can_free_entry {
            return;
        }
        match (*entry).type_0 {
            kSDItemUnknown => xfree((*entry).data.unknown_item.contents.cast()),
            kSDItemHeader => api_free_dict((*entry).data.header),
            kSDItemGlobalMark | kSDItemJump | kSDItemLocalMark | kSDItemChange => {
                xfree((*entry).data.filemark.fname.cast());
            }
            kSDItemSearchPattern => api_free_string((*entry).data.search_pattern.pat),
            kSDItemRegister => {
                let reg = (*entry).data.reg;
                for i in 0..reg.contents_size {
                    api_free_string(*reg.contents.add(i));
                }
                xfree(reg.contents.cast());
            }
            kSDItemHistoryEntry => xfree((*entry).data.history_item.string.cast()),
            kSDItemVariable => {
                xfree((*entry).data.global_var.name.cast());
                tv_clear(&raw mut (*entry).data.global_var.value);
            }
            kSDItemSubString => xfree((*entry).data.sub_string.sub.cast()),
            kSDItemBufferList => {
                let list = (*entry).data.buffer_list;
                for i in 0..list.size {
                    xfree((*list.buffers.add(i)).fname.cast());
                    xfree((*list.buffers.add(i)).additional_data.cast());
                }
                xfree(list.buffers.cast());
            }
            _ => {}
        }
        xfree((*entry).additional_data.cast());
        (*entry).additional_data = core::ptr::null_mut();
    }
}

/// Apply a ShaDa file held in memory rather than on disk. The context stack
/// keeps its registers, jumps, buffer list and variables in this format.
pub unsafe fn shada_read_string(string: String_0, flags: c_int) {
    unsafe {
        if string.size == 0 {
            return;
        }
        let mut sd_reader: FileDescriptor = core::mem::zeroed();
        file_open_buffer(&raw mut sd_reader, string.data, string.size);
        shada_read(&raw mut sd_reader, flags);
        close_file(&raw mut sd_reader);
    }
}
