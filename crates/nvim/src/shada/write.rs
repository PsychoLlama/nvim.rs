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

#![forbid(unsafe_code)]

use crate::ascii::ascii_isdigit;
use crate::buffer::find_buf;
use crate::eval::{get_copy_id, set_ref_in_dict_items, set_ref_in_list_items};
use crate::ex_cmds::shada_replacement;
use crate::mark::{set_last_cursor, shada_buffer_marks, shada_changes, shada_global_marks};
use crate::memory::{XString, qsort_by};
use crate::option::vars::{p_enc, p_hi};
use crate::os::env::os_get_pid;
use crate::os::fileio::WriteFile;
use crate::os::time::os_time;
use crate::search::{search_was_last_used, state::no_hlsearch};
use crate::types::builders::DictBuf;
use crate::types::{
    Integer, Object, String_0, TypVal, VAR_DICT, VAR_FLAVOUR_SHADA, VAR_FUNC, VAR_LIST, VAR_PARTIAL,
};
use crate::version::LONG_VERSION;
use crate::winlayer::{Buf, Win, buffers, tab_windows};
use core::ffi::{c_char, c_int};

use super::collect::{
    VarWalk, find_removable_bufs, hist_type2char, ignore_buf, replace_numbered_mark,
    search_pattern_entry, shada_get_buflist, shada_init_jumps, shada_initialize_registers,
};
use super::file::{find_shada_parameter, get_shada_parameter, shada_removable};
use super::merge::{compare_file_marks, read_when_writing};
use super::pack::{Packer, pack_entry};
use super::unpack::Source;
use super::{
    HIST_COUNT, HIST_SEARCH, HistoryMergerState, ReadKinds, RemovableBufs, ShadaEntry,
    ShadaEntryData, ShadaFileMark, ShadaGlobalVar, WANT_ALWAYS, WANT_UNKNOWN, WriteMergerState,
    WriteStatus, kind, mark_global_index, mark_local_index, want,
};

/// How much packed output is kept before it is handed to the file.
const FLUSH_AT: usize = 64 * 1024;

/// What the `'shada'` option allows a write to contain.
struct Limits {
    /// `s`: entries whose payload is bigger than this many kilobytes are
    /// dropped. Zero means nothing is written at all.
    max_kbyte: usize,
    /// `<`, or `"` when `<` is absent: how many lines of a register are
    /// kept. Zero means no registers.
    max_reg_lines: c_int,
    /// `'`: how many files' local marks are kept. Zero also turns off the
    /// jump list and the change lists.
    num_marked_files: usize,
    /// `!`: whether global variables are written.
    global_vars: bool,
    /// `f`: whether global and numbered marks are written.
    global_marks: bool,
}

impl Limits {
    /// Read `'shada'`. `None` when `s0` says the file may hold nothing.
    fn from_shada_option() -> Option<Limits> {
        let mut max_kbyte_i = get_shada_parameter(c_int::from(b's'));
        if max_kbyte_i < 0 {
            // Not given: the format's own default.
            max_kbyte_i = 10;
        }
        if max_kbyte_i == 0 {
            return None;
        }
        let mut max_reg_lines = get_shada_parameter(c_int::from(b'<'));
        if max_reg_lines < 0 {
            max_reg_lines = get_shada_parameter(c_int::from(b'"'));
        }
        Some(Limits {
            max_kbyte: max_kbyte_i as usize,
            max_reg_lines,
            num_marked_files: get_shada_parameter(c_int::from(b'\'')) as usize,
            global_vars: find_shada_parameter(c_int::from(b'!')).is_some(),
            global_marks: get_shada_parameter(c_int::from(b'f')) != 0,
        })
    }

    /// Whether any register is written.
    fn registers(&self) -> bool {
        self.max_reg_lines != 0
    }
}

/// One ShaDa write in progress.
struct Writing<'a> {
    /// Everything collected so far, and what the merge decides between.
    /// Boxed because it is some hundreds of KiB of mark and jump slots.
    wms: Box<WriteMergerState>,
    /// Buffers whose file is somewhere marks are not kept for.
    removable_bufs: RemovableBufs,
    /// Packed output not yet handed to the file.
    out: Packer,
    file: &'a mut WriteFile,
    limits: Limits,
    /// Which history types are being written.
    histories: [bool; HIST_COUNT],
}

/// Write a ShaDa file to `writer`, merging `reader`'s contents into it when
/// there is an old file to merge.
pub(crate) fn shada_write(writer: &mut WriteFile, reader: Option<&mut Source>) -> WriteStatus {
    let Some(limits) = Limits::from_shada_option() else {
        return WriteStatus::Successful;
    };

    let mut wms = WriteMergerState::new();
    let histories = init_histories(&mut wms);
    let kinds = wanted_kinds(&limits, &histories);

    let mut writing = Writing {
        wms,
        removable_bufs: crate::registry::id_set(),
        out: Packer::default(),
        file: writer,
        limits,
        histories,
    };

    // Recording where the cursor is in every window is what makes the
    // `"` mark right on exit. It also means `:wshada` moves that mark
    // to the cursor, as `:wviminfo` did.
    for wp in tab_windows() {
        set_last_cursor(wp);
    }
    find_removable_bufs(&mut writing.removable_bufs);

    let ret = writing.run(reader, kinds);
    // Whatever is left goes out however the write went; an error here is
    // the close's to report.
    let rest = writing.out.take();
    let _ = writing.file.write(&rest);
    ret
}

/// Start a history merger for each history type `'shada'` keeps something
/// of, and answer which those are.
fn init_histories(wms: &mut WriteMergerState) -> [bool; HIST_COUNT] {
    let mut wanted = [false; HIST_COUNT];
    for (i, wanted) in wanted.iter_mut().enumerate() {
        let mut num_saved = get_shada_parameter(hist_type2char(i as c_int));
        if num_saved == -1 {
            num_saved = p_hi() as c_int;
        }
        if num_saved > 0 {
            *wanted = true;
            wms.hms[i] = Some(HistoryMergerState::new(i as u8, num_saved as usize, false));
        }
    }
    wanted
}

/// The kinds of entry the merging read wants out of the old file. What is
/// not asked for is copied straight through rather than merged.
fn wanted_kinds(limits: &Limits, histories: &[bool; HIST_COUNT]) -> ReadKinds {
    let mut kinds = WANT_ALWAYS | WANT_UNKNOWN;
    if histories.contains(&true) {
        kinds |= want(kind::HISTORY_ENTRY);
    }
    if limits.registers() {
        kinds |= want(kind::REGISTER);
    }
    if limits.global_vars {
        kinds |= want(kind::VARIABLE);
    }
    if limits.global_marks {
        kinds |= want(kind::GLOBAL_MARK);
    }
    if limits.num_marked_files != 0 {
        kinds |= want(kind::LOCAL_MARK) | want(kind::CHANGE);
    }
    kinds
}

impl Writing<'_> {
    /// Collect, merge and pack. A failure to write stops the pass.
    fn run(&mut self, reader: Option<&mut Source>, kinds: ReadKinds) -> WriteStatus {
        if self.write_header() == WriteStatus::Error {
            return WriteStatus::Error;
        }
        if find_shada_parameter(c_int::from(b'%')).is_some()
            && self.write_buflist() == WriteStatus::Error
        {
            return WriteStatus::Error;
        }
        // Variables go out as they are found rather than into `wms`;
        // only their names are kept, so that the merge knows which of
        // the old file's variables have already been written.
        if self.limits.global_vars && self.dump_variables() == WriteStatus::Error {
            return WriteStatus::Error;
        }

        self.collect_jumps();
        self.collect_search_patterns();
        self.collect_global_marks();
        if self.limits.registers() {
            shada_initialize_registers(&mut self.wms, self.limits.max_reg_lines);
        }
        self.collect_buffer_marks();

        // Whatever the old file holds that this Nvim has no opinion
        // about, or a staler one.
        let mut ret = WriteStatus::Successful;
        if let Some(reader) = reader {
            let max_kbyte = self.limits.max_kbyte;
            let read = read_when_writing(reader, kinds, max_kbyte, &mut self.wms, &mut self.out);
            if read != WriteStatus::Successful {
                ret = read;
            }
            if self.flush() == WriteStatus::Error {
                return WriteStatus::Error;
            }
        }

        self.update_numbered_marks();
        if self.pack_everything() == WriteStatus::Error {
            return WriteStatus::Error;
        }
        ret
    }

    /// Hand the packed output to the file once there is enough of it.
    fn flush(&mut self) -> WriteStatus {
        if self.out.bytes().len() < FLUSH_AT {
            return WriteStatus::Successful;
        }
        let bytes = self.out.take();
        if self.file.write(&bytes) < 0 {
            return WriteStatus::Error;
        }
        WriteStatus::Successful
    }

    /// Write one entry, dropping one bigger than `max_kbyte`.
    fn pack(&mut self, entry: &ShadaEntry, max_kbyte: usize) -> WriteStatus {
        let ret = pack_entry(&mut self.out, entry, max_kbyte);
        if self.flush() == WriteStatus::Error {
            return WriteStatus::Error;
        }
        ret
    }

    /// What this Nvim was. Nothing ever reads it back; it is there for
    /// anyone looking at the file by hand.
    fn write_header(&mut self) -> WriteStatus {
        let mut header = DictBuf::<5>::new();
        header
            .insert(c"generator", Object::string(String_0::from_cstr(c"nvim")))
            .insert(
                c"version",
                Object::string(String_0::from_cstr(LONG_VERSION)),
            )
            .insert(
                c"max_kbyte",
                Object::integer(self.limits.max_kbyte as Integer),
            )
            .insert(c"pid", Object::integer(os_get_pid()))
            .insert(
                c"encoding",
                Object::string(p_enc(|value| String_0::from_bytes(value.to_bytes()))),
            );
        let entry = ShadaEntry::new(os_time(), ShadaEntryData::Header(header.dict()));
        self.pack(&entry, 0)
    }

    /// The list of files this Nvim has buffers for, so that a later start
    /// can reopen them.
    fn write_buflist(&mut self) -> WriteStatus {
        let entry = shada_get_buflist(&self.removable_bufs);
        self.pack(&entry, 0)
    }

    /// Every global variable `'shada'` says to keep, written as it is found.
    ///
    /// A container that turns out to be part of a cycle is skipped: the
    /// encoder would not terminate on it.
    fn dump_variables(&mut self) -> WriteStatus {
        let timestamp = os_time();
        for (name, value) in VarWalk::new(VAR_FLAVOUR_SHADA) {
            if !writable_value(&value) {
                continue;
            }
            let entry = ShadaEntry::new(
                timestamp,
                ShadaEntryData::Variable(ShadaGlobalVar { name, value }),
            );
            let ret = self.pack(&entry, self.limits.max_kbyte);
            if ret == WriteStatus::Error {
                return WriteStatus::Error;
            }
            if ret == WriteStatus::Successful {
                let ShadaEntryData::Variable(var) = &entry.data else {
                    unreachable!("the entry just built");
                };
                let key = var.name.as_cstr().to_bytes();
                self.wms.dumped_variables.insert(key.into());
            }
        }
        WriteStatus::Successful
    }

    /// The jump list, as far back as `'shada'` keeps files' marks.
    fn collect_jumps(&mut self) {
        if self.limits.num_marked_files > 0 {
            self.wms.jumps_size = shada_init_jumps(&mut self.wms.jumps, &self.removable_bufs);
        }
    }

    /// The last search and substitute patterns, and the last `:substitute`
    /// replacement string. All three ride on the search history's setting.
    fn collect_search_patterns(&mut self) {
        if !self.histories[usize::from(HIST_SEARCH)] {
            return;
        }
        let highlighted = !(no_hlsearch.get() || find_shada_parameter(c_int::from(b'h')).is_some());
        let last_used = search_was_last_used();

        if let Some(entry) = search_pattern_entry(false, last_used, highlighted) {
            self.wms.search_pattern = entry;
        }
        if let Some(entry) = search_pattern_entry(true, last_used, highlighted) {
            self.wms.sub_search_pattern = entry;
        }
        // An empty replacement string is not worth storing.
        if let Some((sub, timestamp, extra)) = shada_replacement() {
            self.wms.replacement = ShadaEntry {
                timestamp,
                data: ShadaEntryData::SubString(sub),
                extra,
            };
        }
    }

    /// The lettered and numbered global marks.
    ///
    /// A mark on a file no buffer holds still names the file; one on a
    /// buffer that has gone, or that sits on a removable medium, is dropped.
    fn collect_global_marks(&mut self) {
        if !self.limits.global_marks {
            return;
        }
        let mut digit_mark_idx = 0;
        for (name, fm) in shada_global_marks() {
            let Some(fname) = self.mark_fname(fm.fnum, fm.fname) else {
                continue;
            };
            let entry = ShadaEntry {
                timestamp: fm.timestamp,
                data: ShadaEntryData::GlobalMark(ShadaFileMark {
                    name,
                    mark: fm.mark,
                    fname,
                }),
                extra: fm.extra,
            };
            // The ten numbered marks are simply the ten most recent; their
            // names are assigned on the way out.
            if ascii_isdigit(c_int::from(name)) {
                replace_numbered_mark(&mut self.wms, digit_mark_idx, entry);
                digit_mark_idx += 1;
            } else if let Some(idx) = mark_global_index(name) {
                self.wms.global_marks[idx] = entry;
            }
        }
    }

    /// The file name to record a global mark against, or `None` when the
    /// mark is not worth keeping.
    fn mark_fname(&self, fnum: c_int, fname: Option<XString>) -> Option<XString> {
        if fnum == 0 {
            debug_assert!(fname.is_some(), "shada: a mark with no buffer or file");
            return fname.filter(|fname| !shada_removable(fname.as_cstr()));
        }
        let buf = find_buf(fnum)?;
        if self.removable_bufs.contains(&buf.raw().cast_const()) {
            return None;
        }
        buf.name.full().map(XString::from_cstr)
    }

    /// Every buffer's local marks and change list, keyed by file name so
    /// that the merge can find the same file's marks in the old file.
    fn collect_buffer_marks(&mut self) {
        if self.limits.num_marked_files == 0 {
            return;
        }
        for buf in buffers() {
            if !ignore_buf(Some(buf), &self.removable_bufs) {
                self.collect_one_buffer(buf);
            }
        }
    }

    fn collect_one_buffer(&mut self, buffer: Buf) {
        let fname = XString::from_cstr(buffer.name.full().expect("an ignored buffer has a name"));
        let filemarks = self.wms.file_marks_for(fname.as_cstr().to_bytes());

        for (name, fm) in shada_buffer_marks(buffer) {
            filemarks.greatest_timestamp = filemarks.greatest_timestamp.max(fm.timestamp);
            let Some(idx) = mark_local_index(name) else {
                continue;
            };
            filemarks.marks[idx] = ShadaEntry {
                timestamp: fm.timestamp,
                data: ShadaEntryData::LocalMark(ShadaFileMark {
                    name,
                    mark: fm.mark,
                    fname: fname.clone(),
                }),
                extra: fm.extra,
            };
        }

        let changes = shada_changes(buffer);
        filemarks.changes_size = changes.len();
        for (slot, fm) in filemarks.changes.iter_mut().zip(changes) {
            filemarks.greatest_timestamp = filemarks.greatest_timestamp.max(fm.timestamp);
            *slot = ShadaEntry {
                timestamp: fm.timestamp,
                data: ShadaEntryData::Change(ShadaFileMark {
                    name: 0,
                    mark: fm.mark,
                    fname: fname.clone(),
                }),
                extra: fm.extra,
            };
        }
    }

    /// Put the cursor's position in at `'0`, shifting the other numbered
    /// marks down and dropping `'9`.
    fn update_numbered_marks(&mut self) {
        if !self.limits.global_marks
            || ignore_buf(Buf::current_or_none(), &self.removable_bufs)
            || Win::current().w_cursor.lnum == 0
        {
            return;
        }
        let fname = Buf::current().name.full().map(XString::from_cstr);
        let entry = ShadaEntry::new(
            os_time(),
            ShadaEntryData::GlobalMark(ShadaFileMark {
                name: b'0' as c_char,
                mark: Win::current().w_cursor,
                fname: fname.expect("an ignored buffer has a name"),
            }),
        );
        replace_numbered_mark(&mut self.wms, 0, entry);
    }

    /// Write everything the merge left in `wms`, in the order the format
    /// wants it, and release it as it goes.
    fn pack_everything(&mut self) -> WriteStatus {
        let max_kbyte = self.limits.max_kbyte;
        let mut lists: Vec<Vec<ShadaEntry>> = Vec::new();
        let wms = &mut self.wms;
        lists.push(take_all(&mut wms.global_marks));
        lists.push(take_all(&mut wms.numbered_marks));
        lists.push(take_all(&mut wms.registers));
        let jumps_size = wms.jumps_size;
        lists.push(take_all(&mut wms.jumps[..jumps_size]));
        lists.push(vec![
            core::mem::replace(&mut wms.search_pattern, ShadaEntry::MISSING),
            core::mem::replace(&mut wms.sub_search_pattern, ShadaEntry::MISSING),
            core::mem::replace(&mut wms.replacement, ShadaEntry::MISSING),
        ]);
        for entry in lists.iter().flatten().filter(|entry| !entry.is_missing()) {
            if self.pack(entry, max_kbyte) == WriteStatus::Error {
                return WriteStatus::Error;
            }
        }
        drop(lists);
        if self.pack_file_marks() == WriteStatus::Error {
            return WriteStatus::Error;
        }
        self.pack_histories()
    }

    /// Every file's marks, most recently touched file first, as many files
    /// as `'shada'` keeps.
    fn pack_file_marks(&mut self) -> WriteStatus {
        let max_kbyte = self.limits.max_kbyte;
        let files = core::mem::take(&mut self.wms.file_marks);
        // The table's own order, which is the order the files were first
        // marked in. `qsort` is unstable, so it is what decides between the
        // files whose newest mark shares a timestamp -- and the `'N` cut
        // below takes the front of the result. The slots are sorted as
        // pointer-sized indices, as upstream sorted pointers.
        let mut order: Vec<usize> = (0..files.len()).collect();
        qsort_by(&mut order, |&a, &b| {
            compare_file_marks(files[a].greatest_timestamp, files[b].greatest_timestamp)
        });

        let to_dump = order.len().min(self.limits.num_marked_files);
        for &at in &order[..to_dump] {
            let file = &files[at];
            let marks = file.marks.iter().filter(|entry| !entry.is_missing());
            let changes = file.changes[..file.changes_size].iter();
            for entry in marks.chain(changes) {
                if self.pack(entry, max_kbyte) == WriteStatus::Error {
                    return WriteStatus::Error;
                }
            }
            // Marks the old file had for this file that this Nvim has
            // nowhere to put.
            for entry in &file.additional_marks {
                if self.pack(entry, 0) == WriteStatus::Error {
                    return WriteStatus::Error;
                }
            }
        }
        WriteStatus::Successful
    }

    /// The merged histories, oldest entry first.
    fn pack_histories(&mut self) -> WriteStatus {
        let max_kbyte = self.limits.max_kbyte;
        for i in 0..HIST_COUNT {
            if !self.histories[i] {
                continue;
            }
            let Some(mut hms) = self.wms.hms[i].take() else {
                continue;
            };
            hms.insert_whole_neovim_history();
            for entry in hms.hmll.drain() {
                if self.pack(&entry, max_kbyte) == WriteStatus::Error {
                    return WriteStatus::Error;
                }
            }
        }
        WriteStatus::Successful
    }
}

/// The entries of a slot array, taken out and left missing.
fn take_all(slots: &mut [ShadaEntry]) -> Vec<ShadaEntry> {
    slots
        .iter_mut()
        .map(|slot| core::mem::replace(slot, ShadaEntry::MISSING))
        .collect()
}

/// Whether a variable's value can be written at all.
///
/// Functions have no representation in the format, and a container that
/// refers to itself would not terminate the encoder.
fn writable_value(vartv: &TypVal) -> bool {
    match vartv.v_type() {
        VAR_FUNC | VAR_PARTIAL => false,
        // A NULL container holds nothing to loop on.
        VAR_DICT => vartv.dict_shared().is_none_or(|dict| {
            let copy_id = get_copy_id();
            set_ref_in_dict_items(dict, copy_id, None) || copy_id != dict.dv_copy_id
        }),
        VAR_LIST => vartv.list_shared().is_none_or(|list| {
            let copy_id = get_copy_id();
            set_ref_in_list_items(list, copy_id, None) || copy_id != list.lv_copy_id
        }),
        _ => true,
    }
}
