//! Applying a ShaDa file to the running editor.
//!
//! [`shada_read`] walks the entries a source yields and puts each one where
//! it belongs — registers, marks, histories, global variables, the buffer
//! list — subject to the read flags that say which kinds the caller asked
//! for and to the `'shada'` option's limits.
//!
//! Every entry the file yields is *given away* rather than copied: the
//! strings it was read into become the register's, the mark's or the
//! history's, and whatever is not given away drops with the entry.

#![forbid(unsafe_code)]

use crate::buffer::{buflist_new, buflist_setfpos};
use crate::eval::typval::tv_list_alloc;
use crate::eval::var_set_global;
use crate::eval::vars::{get_vim_var_list_handle, set_vim_var_list};
use crate::ex_cmds::{shada_replacement, sub_restore_replacement};
use crate::ex_docmd::set_no_hlsearch;
use crate::mark::{
    ShadaMark, shada_insert_change, shada_insert_jump, shada_restore_last_cursor,
    shada_set_global_mark, shada_set_local_mark,
};
use crate::memory::{ThinCString, XString};
use crate::option::vars::p_hi;
use crate::path::{path_fnamecmp, path_try_shorten_fname};
use crate::register::{ShadaYank, shada_register_timestamp, shada_set_register};
use crate::registry::{IdMap, IdSet, id_map, id_set};
use crate::search::{
    ShadaPattern, set_last_used_pattern, shada_pattern_timestamp, shada_set_pattern,
};
use crate::types::{BlnFlags, ColNr, ListRef, SearchOffset, String_0, TypVal, Vv, kListLenUnknown};
use crate::winlayer::{Buf, BufId, Win, buffers, tab_windows};
use core::ffi::{c_char, c_int};

use super::file::{find_shada_parameter, get_shada_parameter};
use super::unpack::Source;
use super::{
    DEFAULT_SEARCH_PATTERN, HIST_COUNT, HistoryMergerState, MOTION_BLOCKWISE, MOTION_CHARWISE,
    MOTION_LINEWISE, ReadKinds, ReadStatus, ShadaBufferListItem, ShadaEntry, ShadaEntryData,
    ShadaExtra, ShadaFileMark, WANT_ALWAYS, kind, read_flags, want,
};

/// Listed, for the buffers the buffer list restores.
const BLN_LISTED: BlnFlags = 2;

/// The kinds of entry a read with these flags is looking for.
///
/// The caller's flags and the `'shada'` option between them decide this;
/// when they agree on nothing there is no point reading the file at all.
fn wanted_kinds(flags: u32, want_marks: bool, get_old_files: bool) -> ReadKinds {
    let mut kinds: ReadKinds = 0;
    if flags & read_flags::INFO != 0 {
        kinds |= WANT_ALWAYS | want(kind::REGISTER) | want(kind::GLOBAL_MARK);
        if p_hi() != 0 {
            kinds |= want(kind::HISTORY_ENTRY);
        }
        if find_shada_parameter(c_int::from(b'!')).is_some() {
            kinds |= want(kind::VARIABLE);
        }
        // The buffer list is only restored into an Nvim that was not
        // given files to edit.
        if find_shada_parameter(c_int::from(b'%')).is_some() && crate::arglist::argcount() == 0 {
            kinds |= want(kind::BUFFER_LIST);
        }
    }
    if want_marks && get_shada_parameter(c_int::from(b'\'')) > 0 {
        kinds |= want(kind::LOCAL_MARK) | want(kind::CHANGE);
    }
    // File marks are also where `v:oldfiles` comes from.
    if get_old_files {
        kinds |= want(kind::LOCAL_MARK);
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
    oldfiles_list: Option<ListRef>,
    /// The file names already in `oldfiles_list`.
    oldfiles_set: IdSet<Box<[u8]>>,
    /// Buffers whose change list grew; the windows showing them are moved
    /// to the end of it once the whole file has been read.
    cl_bufs: IdSet<BufId>,
    /// File name to the loaded buffer for it, if there is one.
    fname_bufs: FnameBufs,
    /// One merger per history type, when histories are wanted.
    hms: [Option<HistoryMergerState>; HIST_COUNT],
}

/// Read a ShaDa source and apply it.
pub(crate) fn shada_read(source: &mut Source, flags: u32) {
    let force = flags & read_flags::FORCE != 0;
    let mut oldfiles_list = get_vim_var_list_handle(Vv::Oldfiles);
    // `v:oldfiles` is only filled in while it is still empty, so that a
    // second file does not append to the first one's answer.
    let get_old_files = flags & (read_flags::OLDFILES | read_flags::FORCE) != 0
        && (force || oldfiles_list.as_ref().is_none_or(|list| list.is_empty()));
    let want_marks = flags & read_flags::MARKS != 0;

    let kinds = wanted_kinds(flags, want_marks, get_old_files);
    if kinds == 0 {
        return;
    }

    let mut hms = [const { None }; HIST_COUNT];
    if kinds & want(kind::HISTORY_ENTRY) != 0 {
        for (i, hms) in hms.iter_mut().enumerate() {
            *hms = Some(HistoryMergerState::new(i as u8, p_hi() as usize, true));
        }
    }
    if get_old_files && (oldfiles_list.is_none() || force) {
        let held = tv_list_alloc(kListLenUnknown as isize);
        oldfiles_list = Some(held.clone());
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

    loop {
        match source.next_entry(kinds, 0) {
            Ok(entry) => state.apply(entry),
            Err(ReadStatus::Finished) => break,
            // One bad entry is skipped; a file that turned out not to be
            // ShaDa, or that stopped reading, ends the pass with what has
            // been applied so far left in place.
            Err(ReadStatus::Malformed) => continue,
            Err(_) => break,
        }
    }

    state.finish(kinds);
}

impl Reading {
    /// Put one entry from the file where it belongs.
    fn apply(&mut self, entry: ShadaEntry) {
        let ShadaEntry {
            timestamp,
            data,
            extra,
        } = entry;
        match data {
            ShadaEntryData::Missing => unreachable!("shada: read an entry with no type"),
            // Only reached with the unknown kinds asked for, which a plain
            // read never does; the header is never read back.
            ShadaEntryData::Unknown(_) | ShadaEntryData::Header(_) => {}
            ShadaEntryData::SearchPattern(pattern) => {
                self.apply_search_pattern(pattern, timestamp, extra);
            }
            ShadaEntryData::SubString(sub) => self.apply_sub_string(sub, timestamp, extra),
            ShadaEntryData::HistoryEntry(item) => {
                let histtype = usize::from(item.histtype);
                if histtype < HIST_COUNT
                    && let Some(hms) = &mut self.hms[histtype]
                {
                    let entry = ShadaEntry {
                        timestamp,
                        data: ShadaEntryData::HistoryEntry(item),
                        extra,
                    };
                    hms.insert(entry, true);
                }
            }
            ShadaEntryData::Register(reg) => {
                if reg.motion != MOTION_CHARWISE
                    && reg.motion != MOTION_LINEWISE
                    && reg.motion != MOTION_BLOCKWISE
                {
                    return;
                }
                if !self.force
                    && shada_register_timestamp(reg.name).is_none_or(|held| held >= timestamp)
                {
                    return;
                }
                let yank = ShadaYank {
                    contents: reg.contents,
                    motion: reg.motion,
                    width: reg.width as ColNr,
                    timestamp,
                    extra,
                };
                shada_set_register(reg.name, yank, reg.is_unnamed);
            }
            ShadaEntryData::Variable(var) => var_set_global(var.name.as_cstr(), var.value),
            ShadaEntryData::GlobalMark(mark) => {
                let (name, fm) = self.file_mark(mark, timestamp, extra);
                shada_set_global_mark(name, fm, !self.force);
            }
            ShadaEntryData::Jump(mark) => {
                shada_insert_jump(self.file_mark(mark, timestamp, extra).1)
            }
            ShadaEntryData::BufferList(list) => apply_buffer_list(list),
            ShadaEntryData::LocalMark(mark) => {
                self.apply_buffer_mark(mark, timestamp, extra, false);
            }
            ShadaEntryData::Change(mark) => self.apply_buffer_mark(mark, timestamp, extra, true),
        }
    }

    /// A search or substitute pattern. The one this session already has
    /// wins a tie, unless the read was forced.
    fn apply_search_pattern(
        &self,
        mut pat: crate::types::KeyDict__shada_search_pat,
        timestamp: u64,
        extra: Option<ShadaExtra>,
    ) {
        // A key the file left out reads as `DEFAULT_SEARCH_PATTERN`'s: the
        // parser fills those in.
        let default = DEFAULT_SEARCH_PATTERN;
        let flag = |set: Option<bool>, d: Option<bool>| set.or(d).unwrap_or(false);
        let is_sub = flag(pat.is_substitute_pattern, default.is_substitute_pattern);
        if !self.force && shada_pattern_timestamp(is_sub).is_some_and(|held| held >= timestamp) {
            return;
        }
        let pattern = ShadaPattern {
            pat: pat.pat.take().unwrap_or(String_0::NULL),
            magic: flag(pat.magic, default.magic),
            no_scs: !flag(pat.smartcase, default.smartcase),
            timestamp,
            off: SearchOffset {
                dir: if flag(pat.search_backward, default.search_backward) {
                    b'?'
                } else {
                    b'/'
                } as c_char,
                line: flag(pat.has_line_offset, default.has_line_offset),
                end: flag(pat.place_cursor_at_end, default.place_cursor_at_end),
                off: pat.offset.or(default.offset).unwrap_or(0),
            },
            extra,
        };
        shada_set_pattern(is_sub, pattern);
        if flag(pat.is_last_used, default.is_last_used) {
            set_last_used_pattern(is_sub);
            set_no_hlsearch(!flag(pat.highlighted, default.highlighted));
        }
    }

    /// The last `:substitute` replacement string.
    fn apply_sub_string(&self, sub: XString, timestamp: u64, extra: Option<ShadaExtra>) {
        if !self.force && shada_replacement().is_some_and(|(_, held, _)| held >= timestamp) {
            return;
        }
        // The store expands the restored text's `~` once. Without that the
        // restored string is close to useless: `s//~` does not reach it
        // until one has happened. Vim did not do this.
        sub_restore_replacement(sub, timestamp, extra);
    }

    /// A global mark or a jump-list entry, and its name. Both name a *file*,
    /// so a loaded buffer for that name is looked for first: when there is
    /// one the mark refers to it by number and the file name is dropped.
    fn file_mark(
        &mut self,
        mark: ShadaFileMark,
        timestamp: u64,
        extra: Option<ShadaExtra>,
    ) -> (c_char, ShadaMark) {
        let buf = buffer_for_fname(&mut self.fname_bufs, mark.fname.as_cstr().to_bytes());
        let fm = ShadaMark {
            mark: mark.mark,
            fnum: buf.map_or(0, |b| b.handle),
            timestamp,
            fname: buf.is_none().then_some(mark.fname),
            extra,
        };
        (mark.name, fm)
    }

    /// A buffer-local mark or change-list entry.
    ///
    /// These are also what `v:oldfiles` is built from, which is why the file
    /// name matters even when the marks themselves were not asked for.
    fn apply_buffer_mark(
        &mut self,
        mark: ShadaFileMark,
        timestamp: u64,
        extra: Option<ShadaExtra>,
        change: bool,
    ) {
        let fname = mark.fname.as_cstr().to_bytes();
        if self.get_old_files && !self.oldfiles_set.contains(fname) {
            self.oldfiles_set.insert(fname.into());
            if let Some(list) = &mut self.oldfiles_list {
                list.push(TypVal::string(Some(ThinCString::from_cstr(
                    mark.fname.as_cstr(),
                ))));
            }
        }
        if !self.want_marks {
            return;
        }

        // A mark on a file no buffer is holding has nowhere to go.
        let Some(buffer) = buffer_for_fname(&mut self.fname_bufs, fname) else {
            return;
        };
        let fm = ShadaMark {
            mark: mark.mark,
            fnum: buffer.handle,
            timestamp,
            fname: None,
            extra,
        };
        if change {
            self.cl_bufs.insert(buffer.id());
            shada_insert_change(buffer, fm);
        } else {
            shada_set_local_mark(mark.name, buffer, fm, !self.force);
        }
    }

    /// Everything that could only be done once the whole file had been read.
    fn finish(&mut self, kinds: ReadKinds) {
        // The mergers hold both the file's history and Nvim's; folding in
        // what is left of Nvim's makes the merged ring the history.
        if kinds & want(kind::HISTORY_ENTRY) != 0 {
            for hms in &mut self.hms {
                if let Some(mut hms) = hms.take() {
                    hms.insert_whole_neovim_history();
                    hms.into_history();
                }
            }
        }
        // A window showing a buffer whose change list grew sits at the
        // end of it, as if the changes had just been made.
        if !self.cl_bufs.is_empty() {
            for mut wp in tab_windows() {
                if wp
                    .w_buffer
                    .try_id()
                    .is_some_and(|id| self.cl_bufs.contains(&id))
                {
                    wp.w_changelistidx = wp.w_buffer.b_changelistlen;
                }
            }
        }
    }
}

/// The buffer list the file was written with: each name becomes a listed
/// buffer with its cursor where it was left.
fn apply_buffer_list(list: Vec<ShadaBufferListItem>) {
    for item in list {
        let fname = item.fname.as_cstr();
        let sfname = path_try_shorten_fname(fname);
        let Some(mut buf) = buflist_new(Some(fname), Some(sfname), 0, BLN_LISTED as c_int) else {
            continue;
        };
        shada_restore_last_cursor(buf, item.pos);
        let (lnum, col) = (buf.b_last_cursor.mark.lnum, buf.b_last_cursor.mark.col);
        buflist_setfpos(buf, Some(Win::current()), lnum, col, false);
        buf.additional_data = item.extra;
    }
}

/// The loaded buffer editing `fname`, or `None` when there is none.
///
/// Answers are memoised in `fname_bufs`, whose keys are copies.
fn buffer_for_fname(fname_bufs: &mut FnameBufs, fname: &[u8]) -> Option<Buf> {
    if let Some(&memoised) = fname_bufs.get(fname) {
        return memoised.and_then(BufId::get);
    }
    let found = buffers().find(|buf| {
        buf.name
            .full()
            .is_some_and(|full| path_fnamecmp(fname, full.to_bytes()) == 0)
    });
    fname_bufs.insert(fname.into(), found.map(Buf::id));
    found
}

/// Apply a ShaDa file held in memory rather than on disk. The context stack
/// keeps its registers, jumps, buffer list and variables in this format.
pub fn shada_read_string(string: String_0, flags: c_int) {
    if string.is_empty() {
        return;
    }
    let mut source = Source::Bytes {
        data: string.as_bytes(),
        read: 0,
    };
    shada_read(&mut source, flags as u32);
    source.close();
}
