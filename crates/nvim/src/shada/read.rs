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
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use core::ffi::{c_char, c_int, c_uint};

use super::*;
use crate::types::{Vv, kListLenUnknown};
use crate::winlayer::{Buf, BufId, Win};

/// What a mark restored from a file starts its view at: nothing is known
/// about where the window was scrolled to.
const INIT_FMARKV: FileMarkView = FileMarkView {
    topline_offset: MAXLNUM,
    skipcol: 0,
};

/// The kinds of entry a read with these flags is looking for.
///
/// The caller's flags and the `'shada'` option between them decide this;
/// when they agree on nothing there is no point reading the file at all.
fn wanted_kinds(flags: c_int, want_marks: bool, get_old_files: bool) -> c_uint {
    let mut kinds: c_uint = 0;
    if flags & kShaDaWantInfo as c_int != 0 {
        kinds |= kSDReadUndisableableData | kSDReadRegisters | kSDReadGlobalMarks;
        if p_hi() != 0 {
            kinds |= kSDReadHistory;
        }
        if !find_shada_parameter('!' as c_int).is_null() {
            kinds |= kSDReadVariables;
        }
        // The buffer list is only restored into an Nvim that was not
        // given files to edit.
        if !find_shada_parameter('%' as c_int).is_null()
            && unsafe { (*Win::current().w_alist).al_ga.len() as c_int } == 0
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

/// The loaded buffer for each file name asked about, or `None` when there is
/// none. Memoises the walk of the buffer list.
///
/// A **number**, not an address: the map outlives whatever the entries
/// between two lookups do, and a wiped buffer has to read back as gone
/// rather than as whatever the allocator has since put there.
type FnameBufs = IdMap<Box<[u8]>, Option<BufId>>;

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
    oldfiles_list: *mut List,
    /// The file names already in `oldfiles_list`.
    oldfiles_set: IdSet<Box<[u8]>>,
    /// Buffers whose change list grew; the windows showing them are moved
    /// to the end of it once the whole file has been read.
    cl_bufs: IdSet<*mut Buffer>,
    /// File name to the loaded buffer for it, if there is one. Memoises the
    /// walk of the buffer list; the keys are owned copies.
    fname_bufs: FnameBufs,
    /// One merger per history type, used only when histories are wanted.
    hms: [HistoryMergerState; HIST_COUNT as usize],
}

/// Read a ShaDa file and apply it.
///
/// # Safety
///
/// `sd_reader` must point at an open file descriptor, unaliased for the call.
pub(crate) unsafe fn shada_read(sd_reader: *mut FileDescriptor, flags: c_int) {
    let force = flags & kShaDaForceit as c_int != 0;
    let mut oldfiles_list = get_vim_var_list(Vv::Oldfiles);
    // `v:oldfiles` is only filled in while it is still empty, so that a
    // second file does not append to the first one's answer.
    let get_old_files = flags & (kShaDaGetOldfiles | kShaDaForceit) as c_int != 0
        && (force || list_len(unsafe { oldfiles_list.as_ref() }) == 0);
    let want_marks = flags & kShaDaWantMarks as c_int != 0;

    let srni_flags = wanted_kinds(flags, want_marks, get_old_files);
    if srni_flags == 0 {
        return;
    }

    let mut hms = [const { HistoryMergerState::EMPTY }; HIST_COUNT as usize];
    if srni_flags & kSDReadHistory != 0 {
        for (i, hms) in hms.iter_mut().enumerate() {
            unsafe { hms_init(hms, i as uint8_t, p_hi() as size_t, true, true) };
        }
    }
    if get_old_files && (oldfiles_list.is_null() || force) {
        let held = tv_list_alloc(kListLenUnknown as ptrdiff_t);
        oldfiles_list = held.as_ptr();
        set_vim_var_list(Vv::Oldfiles, Some(held));
    }

    let mut state = Reading {
        force,
        want_marks,
        get_old_files,
        oldfiles_list,
        oldfiles_set: id_set(),
        cl_bufs: id_set(),
        fname_bufs: id_map(),
        hms,
    };

    let mut entry = ShadaEntry::MISSING;
    loop {
        match unsafe { shada_read_next_item(sd_reader, &raw mut entry, srni_flags, 0) } {
            kSDReadStatusSuccess => {}
            kSDReadStatusFinished => break,
            // One bad entry is skipped; a file that turned out not to be
            // ShaDa, or that stopped reading, ends the pass with what has
            // been applied so far left in place.
            kSDReadStatusMalformed => continue,
            _ => break,
        }
        // The entry moves to `apply`, which owns it from here; the next
        // pass refills the slot from the file.
        state.apply(core::mem::replace(&mut entry, ShadaEntry::MISSING));
    }

    state.finish(srni_flags);
}

impl Reading {
    /// Put one entry from the file where it belongs.
    fn apply(&mut self, mut entry: ShadaEntry) {
        // On the kind rather than the payload: every arm below hands the
        // whole entry on, and reads its payload out of it there.
        match entry.kind() {
            kSDItemMissing => unreachable!("shada: read an entry with no type"),
            // Only reached with `kSDReadUnknown`, which a plain read
            // never asks for.
            kSDItemUnknown => {}
            kSDItemHeader => unsafe { shada_free_shada_entry(&raw mut entry) },
            kSDItemSearchPattern => self.apply_search_pattern(entry),
            kSDItemSubString => self.apply_sub_string(entry),
            kSDItemHistoryEntry => self.apply_history(entry),
            kSDItemRegister => self.apply_register(entry),
            kSDItemVariable => apply_variable(entry),
            kSDItemGlobalMark | kSDItemJump => self.apply_file_mark(entry),
            kSDItemBufferList => apply_buffer_list(entry),
            kSDItemLocalMark | kSDItemChange => self.apply_buffer_mark(entry),
            other => unreachable!("shada: entry type {other} has no reader"),
        }
    }

    /// A search or substitute pattern. The one this session already has
    /// wins a tie, unless the read was forced.
    fn apply_search_pattern(&self, mut entry: ShadaEntry) {
        // A key the file left out reads as `DEFAULT_SEARCH_PATTERN`'s: the
        // parser fills those in, and a collector names every one of them, so
        // in practice nothing here falls back -- the defaults are what say
        // so.
        let pat = entry.data.search_pattern_mut();
        let default = DEFAULT_SEARCH_PATTERN;
        let flag = |set: Option<bool>, d: Option<bool>| set.or(d).unwrap_or(false);
        let is_sub = flag(pat.is_substitute_pattern, default.is_substitute_pattern);
        if !self.force {
            let mut current: SearchPattern = unsafe { core::mem::zeroed() };
            if is_sub {
                unsafe { get_substitute_pattern(&raw mut current) };
            } else {
                unsafe { get_search_pattern(&raw mut current) };
            }
            if !current.pat.is_null() && current.timestamp >= entry.timestamp {
                unsafe { shada_free_shada_entry(&raw mut entry) };
                return;
            }
        }

        // The pattern takes the entry's string and extra data over, so the
        // string gives its block up rather than releasing it here.
        let text = pat.pat.take().unwrap_or(String_0::NULL);
        let patlen = text.len();
        let spat = SearchPattern {
            pat: text.into_raw(),
            patlen,
            magic: flag(pat.magic, default.magic),
            no_scs: !flag(pat.smartcase, default.smartcase),
            timestamp: entry.timestamp,
            off: SearchOffset {
                dir: if flag(pat.search_backward, default.search_backward) {
                    b'?'
                } else {
                    b'/'
                } as c_char,
                line: flag(pat.has_line_offset, default.has_line_offset),
                end: flag(pat.place_cursor_at_end, default.place_cursor_at_end),
                off: pat.offset.or(default.offset).unwrap_or(0) as int64_t,
            },
            additional_data: entry.additional_data,
        };
        if is_sub {
            unsafe { set_substitute_pattern(spat) };
        } else {
            unsafe { set_search_pattern(spat) };
        }
        if flag(pat.is_last_used, default.is_last_used) {
            set_last_used_pattern(is_sub);
            set_no_hlsearch(!flag(pat.highlighted, default.highlighted));
        }
    }

    /// The last `:substitute` replacement string.
    fn apply_sub_string(&self, mut entry: ShadaEntry) {
        let sub_string = *entry.data.sub_string_mut();
        if !self.force {
            let mut current: SubReplacementString = unsafe { core::mem::zeroed() };
            unsafe { sub_get_replacement(&raw mut current) };
            if !current.sub.is_null() && current.timestamp >= entry.timestamp {
                unsafe { shada_free_shada_entry(&raw mut entry) };
                return;
            }
        }
        let sub = sub_string.sub;
        unsafe {
            sub_set_replacement(SubReplacementString {
                sub,
                timestamp: entry.timestamp,
                additional_data: entry.additional_data,
            })
        };
        // Without a `regtilde` call the restored string is close to
        // useless: `s//~` does not reach it until one has happened. Vim
        // did not do this.
        unsafe { regtilde(sub, magic_isset() as c_int, false) };
    }

    /// One history entry, handed to the merger for its type.
    fn apply_history(&mut self, mut entry: ShadaEntry) {
        let histtype = entry.data.history().histtype as c_uint;
        if histtype >= HIST_COUNT {
            unsafe { shada_free_shada_entry(&raw mut entry) };
            return;
        }
        unsafe { hms_insert(&raw mut self.hms[histtype as usize], entry, true) };
    }

    /// One register. The register this session already holds wins a tie,
    /// unless the read was forced.
    fn apply_register(&self, mut entry: ShadaEntry) {
        let reg = *entry.data.register_mut();
        if reg.type_0 != kMTCharWise && reg.type_0 != kMTLineWise && reg.type_0 != kMTBlockWise {
            unsafe { shada_free_shada_entry(&raw mut entry) };
            return;
        }
        if !self.force {
            let current = unsafe { op_reg_get(reg.name) };
            if current.is_null() || unsafe { (*current).timestamp } >= entry.timestamp {
                unsafe { shada_free_shada_entry(&raw mut entry) };
                return;
            }
        }
        let yank = YankReg {
            y_array: reg.contents,
            y_size: reg.contents_size,
            y_type: reg.type_0,
            y_width: reg.width as ColNr,
            timestamp: entry.timestamp,
            additional_data: entry.additional_data,
        };
        let stored = unsafe { op_reg_set(reg.name, yank, reg.is_unnamed) };
        if !stored {
            unsafe { shada_free_shada_entry(&raw mut entry) };
        }
    }

    /// A global mark or a jump-list entry. Both name a *file*, so a loaded
    /// buffer for that name is looked for first: when there is one the mark
    /// refers to it by number and the file name is dropped.
    fn apply_file_mark(&mut self, mut entry: ShadaEntry) {
        let buf = unsafe { buffer_for_fname(&mut self.fname_bufs, entry.data.filemark().fname) };
        if buf.is_some() {
            unsafe { xfree(entry.data.filemark().fname.cast()) };
            entry.data.filemark_mut().fname = core::ptr::null_mut();
        }
        let fm = XFileMark {
            fmark: FileMark {
                mark: entry.data.filemark().mark,
                fnum: buf.map_or(0, |b| b.handle) as c_int,
                timestamp: entry.timestamp,
                view: INIT_FMARKV,
                additional_data: entry.additional_data,
            },
            // Null exactly when a buffer was found, from just above.
            fname: entry.data.filemark().fname,
        };
        if let ShadaEntryData::GlobalMark(mark) = entry.data {
            if !unsafe { mark_set_global(mark.name, fm, !self.force) } {
                unsafe { shada_free_shada_entry(&raw mut entry) };
            }
            return;
        }
        insert_jump(fm, buf, entry);
    }

    /// A buffer-local mark or change-list entry.
    ///
    /// These are also what `v:oldfiles` is built from, which is why the file
    /// name matters even when the marks themselves were not asked for.
    fn apply_buffer_mark(&mut self, mut entry: ShadaEntry) {
        // SAFETY: an entry's file name is null or NUL-terminated.
        let seen = self
            .oldfiles_set
            .contains(unsafe { shada_key(entry.data.filemark().fname) });
        if self.get_old_files && !seen {
            // The entry's own string can be handed to the list, unless
            // the mark below is still going to need it.
            let fname = if self.want_marks {
                unsafe { xstrdup(entry.data.filemark().fname) }
            } else {
                entry.data.filemark().fname
            };
            // SAFETY: `fname` is null or NUL-terminated.
            self.oldfiles_set.insert(unsafe { shada_key(fname) }.into());
            unsafe { (*self.oldfiles_list).push_allocated_string(fname) };
            if !self.want_marks {
                entry.data.filemark_mut().fname = core::ptr::null_mut();
            }
        }
        if !self.want_marks {
            unsafe { shada_free_shada_entry(&raw mut entry) };
            return;
        }

        // A mark on a file no buffer is holding has nowhere to go.
        let buf = unsafe { buffer_for_fname(&mut self.fname_bufs, entry.data.filemark().fname) };
        let Some(buffer) = buf else {
            unsafe { shada_free_shada_entry(&raw mut entry) };
            return;
        };
        let fm = FileMark {
            mark: entry.data.filemark().mark,
            fnum: buffer.handle as c_int,
            timestamp: entry.timestamp,
            view: INIT_FMARKV,
            additional_data: entry.additional_data,
        };
        if let ShadaEntryData::LocalMark(mark) = entry.data {
            if !unsafe { mark_set_local(mark.name, buffer, fm, !self.force) } {
                unsafe { shada_free_shada_entry(&raw mut entry) };
                return;
            }
        } else {
            self.cl_bufs.insert(buffer.raw());
            insert_change(buffer, fm);
        }
        // The mark took the extra data; only the file name is left.
        unsafe { xfree(entry.data.filemark().fname.cast()) };
    }

    /// Everything that could only be done once the whole file had been read.
    fn finish(&mut self, srni_flags: c_uint) {
        // SAFETY: the editor's own state, on the main thread.
        // The mergers hold both the file's history and Nvim's; folding
        // in what is left of Nvim's makes the merged ring the history.
        if srni_flags & kSDReadHistory != 0 {
            for hms in &mut self.hms {
                unsafe { hms_insert_whole_neovim_history(hms) };
                unsafe { hms_to_history(hms) };
                unsafe { hms_dealloc(hms) };
            }
        }
        // A window showing a buffer whose change list grew sits at the
        // end of it, as if the changes had just been made.
        if !self.cl_bufs.is_empty() {
            for mut wp in tab_windows() {
                if self.cl_bufs.contains(&wp.w_buffer) {
                    wp.w_changelistidx = unsafe { (*wp.w_buffer).b_changelistlen };
                }
            }
        }
        // The three tables own their keys and are released by `Reading`'s
        // own drop; the buffers `fname_bufs` points at, and the names
        // `oldfiles_set` copied, belong elsewhere.
    }
}

/// A global variable. `var_set_global` takes the value over, so the entry
/// is emptied of it before the rest is freed.
fn apply_variable(mut entry: ShadaEntry) {
    let var = entry.data.variable_mut();
    // The value moves into the variable; the name stays the entry's.
    unsafe { var_set_global(var.name, var.value.take()) };
    unsafe { shada_free_shada_entry(&raw mut entry) };
}

/// The buffer list the file was written with: each name becomes a listed
/// buffer with its cursor where it was left.
fn apply_buffer_list(mut entry: ShadaEntry) {
    let list = entry.data.buffer_list();
    for i in 0..list.size {
        let item = unsafe { list.buffers.add(i) };
        let sfname = unsafe { path_try_shorten_fname((*item).fname) };
        let buf = unsafe { buflist_new((*item).fname, sfname, 0, BLN_LISTED as c_int) };
        let Some(mut buf) = buf else {
            continue;
        };
        unsafe { free_fmark(buf.b_last_cursor.clone()) };
        let cursor = FileMark {
            mark: unsafe { (*item).pos },
            fnum: 0,
            timestamp: os_time(),
            view: INIT_FMARKV,
            additional_data: core::ptr::null_mut(),
        };
        buf.b_last_cursor = cursor;
        let (lnum, col) = (buf.b_last_cursor.mark.lnum, buf.b_last_cursor.mark.col);
        buflist_setfpos(buf, Some(Win::current()), lnum, col, false);
        unsafe { xfree(buf.additional_data.cast()) };
        buf.additional_data = unsafe { (*item).additional_data };
        unsafe { (*item).additional_data = core::ptr::null_mut() };
    }
    unsafe { shada_free_shada_entry(&raw mut entry) };
}

/// The loaded buffer editing `fname`, or null when there is none.
///
/// Answers are memoised in `fname_bufs`, whose keys are copies this makes
/// and the caller frees.
///
/// # Safety
///
/// `fname` must point at a NUL-terminated string.
unsafe fn buffer_for_fname(fname_bufs: &mut FnameBufs, fname: *const c_char) -> Option<Buf> {
    // SAFETY: the caller's file name, null or NUL-terminated.
    let key = unsafe { shada_key(fname) };
    if let Some(&memoised) = fname_bufs.get(key) {
        return memoised.and_then(BufId::get);
    }
    let mut found = None;
    for buf in buffers() {
        // SAFETY: `fname` and the buffer's own name are both C strings.
        if !buf.b_ffname.is_null()
            && unsafe { path_fnamecmp(cstr::at(fname), cstr::at(buf.b_ffname)) } == 0
        {
            found = Some(buf);
            break;
        }
    }
    fname_bufs.insert(key.into(), found.map(Buf::id));
    found
}

/// Put a jump into `curwin`'s jump list, which is kept oldest first.
///
/// A jump the list already holds — same position, same file — is dropped
/// rather than inserted twice, and so is one older than a list that is
/// already full.
fn insert_jump(fm: XFileMark, buffer: Option<Buf>, mut entry: ShadaEntry) {
    let mut win = Win::current();
    let mut i = win.w_jumplistlen;
    while i > 0 {
        let existing = &win.w_jumplist[i as usize - 1];
        if existing.fmark.timestamp <= fm.fmark.timestamp {
            let same_file = if buffer.is_none() {
                // SAFETY: both names are NUL-terminated: the list's own, and
                // the caller's, which it promised.
                !existing.fname.is_null() && unsafe { cstr::eq(fm.fname, existing.fname) }
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
    if i > 0 && win.w_jumplistlen == JUMPLISTSIZE {
        // SAFETY: the oldest jump is about to be overwritten, so what it
        // holds is this call's to release.
        unsafe { free_xfmark(win.w_jumplist[0].clone()) };
    }
    let len = win.w_jumplistlen;
    let i = marklist_insert(&mut win.w_jumplist, len, i);
    if i == -1 {
        // SAFETY: the entry was read from the file, so it owns its strings.
        unsafe { shada_free_shada_entry(&raw mut entry) };
        return;
    }
    win.w_jumplist[i as usize] = fm;
    if win.w_jumplistlen < JUMPLISTSIZE {
        win.w_jumplistlen += 1;
    }
    // Keep the cursor into the list pointing at the same jump.
    if win.w_jumplistidx >= i && win.w_jumplistidx < win.w_jumplistlen {
        win.w_jumplistidx += 1;
    }
}

/// [`insert_jump`] for a buffer's change list, which needs no file name to
/// compare on because every entry in it is in this buffer.
fn insert_change(mut buffer: Buf, fm: FileMark) {
    let mut i = buffer.b_changelistlen;
    while i > 0 {
        let existing = &buffer.b_changelist[i as usize - 1];
        if existing.timestamp <= fm.timestamp {
            if marks_equal(existing.mark, fm.mark) {
                i = -1;
            }
            break;
        }
        i -= 1;
    }
    if i > 0 && buffer.b_changelistlen == JUMPLISTSIZE {
        // SAFETY: the oldest change is about to be overwritten, so what it
        // holds is this call's to release.
        unsafe { free_fmark(buffer.b_changelist[0].clone()) };
    }
    let len = buffer.b_changelistlen;
    let i = marklist_insert(&mut buffer.b_changelist, len, i);
    if i == -1 {
        // SAFETY: the mark was read from the file, so it owns its extras.
        unsafe { xfree(fm.additional_data.cast()) };
        return;
    }
    buffer.b_changelist[i as usize] = fm;
    if buffer.b_changelistlen < JUMPLISTSIZE {
        buffer.b_changelistlen += 1;
    }
}

/// Release what an entry read from a file holds.
///
/// Entries built from Nvim's own state borrow their strings from it and say
/// so with `can_free_entry`; those are left alone.
///
/// # Safety
///
/// `entry` must point at an initialized ShaDa entry, unaliased for the call.
pub(crate) unsafe fn shada_free_shada_entry(entry: *mut ShadaEntry) {
    if entry.is_null() || !unsafe { (*entry).can_free_entry } {
        return;
    }
    let data = unsafe { &mut (*entry).data };
    match data {
        ShadaEntryData::Missing => {}
        ShadaEntryData::Unknown(item) => unsafe { xfree(item.contents.cast()) },
        ShadaEntryData::Header(header) => drop(core::mem::take(header)),
        ShadaEntryData::GlobalMark(mark)
        | ShadaEntryData::Jump(mark)
        | ShadaEntryData::LocalMark(mark)
        | ShadaEntryData::Change(mark) => unsafe { xfree(mark.fname.cast()) },
        ShadaEntryData::SearchPattern(pattern) => pattern.pat = None,
        ShadaEntryData::Register(reg) => {
            for i in 0..reg.contents_size {
                // SAFETY: the register owns `contents_size` strings at
                // `contents`, each of them initialised by `parse_register`.
                unsafe { reg.contents.add(i).drop_in_place() };
            }
            unsafe { xfree(reg.contents.cast()) };
        }
        ShadaEntryData::HistoryEntry(item) => unsafe { xfree(item.string.cast()) },
        ShadaEntryData::Variable(var) => {
            unsafe { xfree(var.name.cast()) };
            tv_clear(&mut var.value);
        }
        ShadaEntryData::SubString(sub) => unsafe { xfree(sub.sub.cast()) },
        ShadaEntryData::BufferList(list) => {
            for i in 0..list.size {
                unsafe { xfree((*list.buffers.add(i)).fname.cast()) };
                unsafe { xfree((*list.buffers.add(i)).additional_data.cast()) };
            }
            unsafe { xfree(list.buffers.cast()) };
        }
    }
    unsafe { xfree((*entry).additional_data.cast()) };
    unsafe { (*entry).additional_data = core::ptr::null_mut() };
}

/// Apply a ShaDa file held in memory rather than on disk. The context stack
/// keeps its registers, jumps, buffer list and variables in this format.
///
/// # Safety
///
/// `string` must be a well-formed API string: `size` readable bytes with a
/// NUL at `data[size]`.
pub unsafe fn shada_read_string(string: String_0, flags: c_int) {
    if string.is_empty() {
        return;
    }
    let mut sd_reader: FileDescriptor = unsafe { core::mem::zeroed() };
    unsafe { file_open_buffer(&raw mut sd_reader, string.data(), string.len()) };
    unsafe { shada_read(&raw mut sd_reader, flags) };
    unsafe { close_file(&raw mut sd_reader) };
}
