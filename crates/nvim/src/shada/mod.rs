//! ShaDa: the file that carries registers, marks, histories, variables and
//! the buffer list from one Nvim session to the next.
//!
//! The children split the work by direction. `unpack` reads entries off the
//! wire and `parse` turns each payload into a [`ShadaEntry`]; `read` applies
//! them to the editor. `collect` gathers the editor's state into entries,
//! `merge` decides between those and what the old file holds, `write` drives
//! the pass and `pack` turns entries back into msgpack. `file` is the file
//! itself: where it lives, and the temporary-then-rename dance a write does.
//!
//! Every entry owns what it holds. One read from a file owns the strings it
//! was parsed into; one collected from the editor holds copies, so a merge
//! can drop, replace or move any entry without asking where it came from.

#![forbid(unsafe_code)]

use crate::memory::XString;
use crate::registry::{IdMap, IdSet, id_map, id_set};
use crate::types::Buffer;
use crate::types::{
    ApiDict, KeyDict__shada_search_pat, MotionType, Pos, String_0, Timestamp, TypVal,
};
use core::ffi::c_char;
use std::collections::VecDeque;
use std::ffi::CString;

mod file;
pub use self::file::*;
mod merge;
mod pack;
mod parse;
mod unpack;
pub(crate) use self::merge::marklist_insert;
mod collect;
pub use self::collect::{
    shada_encode_buflist, shada_encode_gvars, shada_encode_jumps, shada_encode_regs,
};
mod read;
pub use self::read::shada_read_string;
mod write;

/// A ShaDa entry's extra data: the map keys or array elements of an entry
/// that this Nvim has no field for, kept as the msgpack they arrived as so
/// that the next write can put them back verbatim.
///
/// `items` counts the keys (each with its value) or elements; `bytes` is
/// their encodings, back to back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShadaExtra {
    items: u32,
    bytes: Vec<u8>,
}

impl ShadaExtra {
    /// Extra data of `items` items encoded as `bytes`.
    pub(crate) fn from_parts(items: u32, bytes: Vec<u8>) -> ShadaExtra {
        ShadaExtra { items, bytes }
    }

    /// How many keys or elements this holds.
    pub(crate) fn items(&self) -> u32 {
        self.items
    }

    /// Their encodings, back to back.
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Append one more item's encoding, starting the extra data on the
    /// first one.
    pub(crate) fn push(extra: &mut Option<ShadaExtra>, item: &[u8]) {
        let extra = extra.get_or_insert_with(|| ShadaExtra::from_parts(0, Vec::new()));
        extra.items += 1;
        extra.bytes.extend_from_slice(item);
    }
}

/// What a ShaDa file holds, by the number the format writes for each kind.
pub(crate) mod kind {
    pub(crate) const HEADER: u64 = 1;
    pub(crate) const SEARCH_PATTERN: u64 = 2;
    pub(crate) const SUB_STRING: u64 = 3;
    pub(crate) const HISTORY_ENTRY: u64 = 4;
    pub(crate) const REGISTER: u64 = 5;
    pub(crate) const VARIABLE: u64 = 6;
    pub(crate) const GLOBAL_MARK: u64 = 7;
    pub(crate) const JUMP: u64 = 8;
    pub(crate) const BUFFER_LIST: u64 = 9;
    pub(crate) const LOCAL_MARK: u64 = 10;
    pub(crate) const CHANGE: u64 = 11;
    /// The largest kind this Nvim knows. Anything above it is an unknown
    /// entry, kept only to be written back out.
    pub(crate) const LAST: u64 = CHANGE;
}

/// Which kinds of entry a read wants, one bit per kind (`1 << kind`), plus
/// one for the kinds this Nvim does not know.
pub(crate) type ReadKinds = u32;

/// The bit for one known kind.
pub(crate) const fn want(kind: u64) -> ReadKinds {
    1 << kind
}

/// Entries of a kind this Nvim does not know.
pub(crate) const WANT_UNKNOWN: ReadKinds = 1 << 12;
/// The kinds no `'shada'` setting turns off: the two patterns and the jump
/// list.
pub(crate) const WANT_ALWAYS: ReadKinds =
    want(kind::SEARCH_PATTERN) | want(kind::SUB_STRING) | want(kind::JUMP);

/// What a read asked for, in the `flags` [`shada_read_everything`] builds.
pub(crate) mod read_flags {
    pub(crate) const INFO: u32 = 1;
    pub(crate) const MARKS: u32 = 2;
    pub(crate) const FORCE: u32 = 4;
    pub(crate) const OLDFILES: u32 = 8;
    pub(crate) const MISSING_ERROR: u32 = 16;
}

/// How reading one entry went.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ReadStatus {
    /// The end of the file.
    Finished,
    /// The file could not be read.
    ReadError,
    /// The file is not ShaDa at all.
    NotShada,
    /// One entry was malformed; the rest may still be good.
    Malformed,
}

/// How writing went.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum WriteStatus {
    Successful,
    /// The old file turned out not to be ShaDa, so it is left alone.
    ReadNotShada,
    /// The file could not be written.
    Error,
    /// One entry could not be written; the rest of the file is still good.
    IgnError,
}

/// The number of history types.
pub(crate) const HIST_COUNT: usize = 5;
/// The search history's number: the only one with a separator.
pub(crate) const HIST_SEARCH: u8 = 1;

/// The three motion types a register can have.
pub(crate) const MOTION_CHARWISE: MotionType = 0;
pub(crate) const MOTION_LINEWISE: MotionType = 1;
pub(crate) const MOTION_BLOCKWISE: MotionType = 2;

/// The lettered marks of one buffer.
pub(crate) const NMARKS: usize = 26;
/// The most entries a jump list or a change list holds.
pub(crate) const JUMPLISTSIZE: usize = 100;

/// One ShaDa entry: when it was made, what it holds, and the extra data it
/// carried that this Nvim has no field for.
pub struct ShadaEntry {
    pub(crate) timestamp: Timestamp,
    pub(crate) data: ShadaEntryData,
    pub(crate) extra: Option<ShadaExtra>,
}

impl ShadaEntry {
    /// An empty slot: what a merger's arrays are filled with.
    pub(crate) const MISSING: ShadaEntry = ShadaEntry {
        timestamp: 0,
        data: ShadaEntryData::Missing,
        extra: None,
    };

    /// An entry with no extra data.
    pub(crate) fn new(timestamp: Timestamp, data: ShadaEntryData) -> ShadaEntry {
        ShadaEntry {
            timestamp,
            data,
            extra: None,
        }
    }

    /// Which kind of entry this is — the number the format writes for it.
    /// An unknown entry answers the kind it arrived as.
    pub(crate) fn kind(&self) -> u64 {
        self.data.kind()
    }

    /// Whether this is an empty slot.
    pub(crate) fn is_missing(&self) -> bool {
        matches!(self.data, ShadaEntryData::Missing)
    }
}

/// One entry's payload, and by which variant it is, the kind of entry.
pub enum ShadaEntryData {
    /// No entry at all: an empty slot in a merger's arrays.
    Missing,
    Header(ApiDict),
    SearchPattern(KeyDict__shada_search_pat),
    /// The last `:substitute` replacement string.
    SubString(XString),
    HistoryEntry(ShadaHistoryItem),
    Register(ShadaRegister),
    Variable(ShadaGlobalVar),
    GlobalMark(ShadaFileMark),
    Jump(ShadaFileMark),
    BufferList(Vec<ShadaBufferListItem>),
    LocalMark(ShadaFileMark),
    Change(ShadaFileMark),
    /// An entry of a kind this Nvim does not know, kept byte for byte so
    /// that writing the file back does not lose it.
    Unknown(ShadaUnknownItem),
}

impl ShadaEntryData {
    /// The number the format writes for this kind of entry.
    pub(crate) fn kind(&self) -> u64 {
        match self {
            ShadaEntryData::Missing => 0,
            ShadaEntryData::Header(_) => kind::HEADER,
            ShadaEntryData::SearchPattern(_) => kind::SEARCH_PATTERN,
            ShadaEntryData::SubString(_) => kind::SUB_STRING,
            ShadaEntryData::HistoryEntry(_) => kind::HISTORY_ENTRY,
            ShadaEntryData::Register(_) => kind::REGISTER,
            ShadaEntryData::Variable(_) => kind::VARIABLE,
            ShadaEntryData::GlobalMark(_) => kind::GLOBAL_MARK,
            ShadaEntryData::Jump(_) => kind::JUMP,
            ShadaEntryData::BufferList(_) => kind::BUFFER_LIST,
            ShadaEntryData::LocalMark(_) => kind::LOCAL_MARK,
            ShadaEntryData::Change(_) => kind::CHANGE,
            ShadaEntryData::Unknown(item) => item.kind,
        }
    }

    /// The mark a global mark, local mark, jump or change entry carries.
    pub(crate) fn filemark(&self) -> &ShadaFileMark {
        match self {
            ShadaEntryData::GlobalMark(mark)
            | ShadaEntryData::Jump(mark)
            | ShadaEntryData::LocalMark(mark)
            | ShadaEntryData::Change(mark) => mark,
            other => unreachable!("shada: entry type {} carries no mark", other.kind()),
        }
    }

    /// [`Self::filemark`], to write to.
    pub(crate) fn filemark_mut(&mut self) -> &mut ShadaFileMark {
        match self {
            ShadaEntryData::GlobalMark(mark)
            | ShadaEntryData::Jump(mark)
            | ShadaEntryData::LocalMark(mark)
            | ShadaEntryData::Change(mark) => mark,
            other => unreachable!("shada: entry type {} carries no mark", other.kind()),
        }
    }

    /// The line a history entry carries.
    pub(crate) fn history(&self) -> &ShadaHistoryItem {
        match self {
            ShadaEntryData::HistoryEntry(item) => item,
            other => unreachable!("shada: entry type {} is not a history entry", other.kind()),
        }
    }
}

/// One buffer of the buffer list: its file and where its cursor was.
pub struct ShadaBufferListItem {
    pub(crate) pos: Pos,
    pub(crate) fname: XString,
    pub(crate) extra: Option<ShadaExtra>,
}

/// An entry of a kind this Nvim does not know.
pub struct ShadaUnknownItem {
    /// The kind it arrived as.
    pub(crate) kind: u64,
    /// Its payload, as it arrived.
    pub(crate) contents: Vec<u8>,
}

/// One global variable.
pub struct ShadaGlobalVar {
    pub(crate) name: XString,
    pub(crate) value: TypVal,
}

/// One register: its name, how it is put back, and its lines.
pub struct ShadaRegister {
    pub(crate) name: c_char,
    pub(crate) motion: MotionType,
    pub(crate) contents: Vec<String_0>,
    pub(crate) is_unnamed: bool,
    /// The width of a blockwise register; 0 for the other two.
    pub(crate) width: usize,
}

/// One history line.
pub struct ShadaHistoryItem {
    pub(crate) histtype: u8,
    pub(crate) text: CString,
    /// The character the search was started with; search history only.
    pub(crate) sep: u8,
}

/// A global mark, local mark, jump or change.
pub struct ShadaFileMark {
    /// The mark's letter; NUL for a jump or a change.
    pub(crate) name: c_char,
    pub(crate) mark: Pos,
    pub(crate) fname: XString,
}

/// Merging one history type: a ring of the newest entries, and the part of
/// Nvim's own history not yet folded in.
pub(crate) struct HistoryMergerState {
    pub(crate) hmll: merge::HistoryRing,
    /// Nvim's own history (oldest first), taken when the merger started and
    /// drained as the file's entries interleave with it.
    pub(crate) pending: VecDeque<ShadaEntry>,
    pub(crate) history_type: u8,
}

/// Everything collected for one write, and what the merge decides between.
pub(crate) struct WriteMergerState {
    pub(crate) hms: [Option<HistoryMergerState>; HIST_COUNT],
    pub(crate) global_marks: [ShadaEntry; 26],
    pub(crate) numbered_marks: [ShadaEntry; 10],
    pub(crate) registers: [ShadaEntry; 37],
    /// The jump list, oldest first; `jumps_size` of them are real.
    pub(crate) jumps: [ShadaEntry; JUMPLISTSIZE],
    pub(crate) jumps_size: usize,
    pub(crate) search_pattern: ShadaEntry,
    pub(crate) sub_search_pattern: ShadaEntry,
    pub(crate) replacement: ShadaEntry,
    /// The names of the variables already written. Membership only.
    pub(crate) dumped_variables: IdSet<Box<[u8]>>,
    /// Where in `file_marks` each file's marks are, by file name.
    pub(crate) file_marks_index: IdMap<Box<[u8]>, usize>,
    /// One file's marks per entry, in the order the files were first seen.
    ///
    /// That order is the input permutation `qsort` is handed when the files
    /// are ordered by their newest mark, and `qsort` is unstable: it is what
    /// decides between files whose newest marks share a timestamp, and the
    /// `'N` cut takes the front of the result. khash was insertion-ordered;
    /// keeping this order is what keeps the port's input the array upstream
    /// sorts.
    pub(crate) file_marks: Vec<Box<FileMarks>>,
}

impl WriteMergerState {
    /// Nothing collected yet.
    pub(crate) fn new() -> Box<WriteMergerState> {
        Box::new(WriteMergerState {
            hms: [const { None }; HIST_COUNT],
            global_marks: [const { ShadaEntry::MISSING }; 26],
            numbered_marks: [const { ShadaEntry::MISSING }; 10],
            registers: [const { ShadaEntry::MISSING }; 37],
            jumps: [const { ShadaEntry::MISSING }; JUMPLISTSIZE],
            jumps_size: 0,
            search_pattern: ShadaEntry::MISSING,
            sub_search_pattern: ShadaEntry::MISSING,
            replacement: ShadaEntry::MISSING,
            dumped_variables: id_set(),
            file_marks_index: id_map(),
            file_marks: Vec::new(),
        })
    }

    /// The slot one file's marks collect into, made on first use.
    pub(crate) fn file_marks_for(&mut self, fname: &[u8]) -> &mut FileMarks {
        let at = match self.file_marks_index.get(fname) {
            Some(&at) => at,
            None => {
                let at = self.file_marks.len();
                self.file_marks.push(FileMarks::new());
                self.file_marks_index.insert(fname.into(), at);
                at
            }
        };
        &mut self.file_marks[at]
    }
}

/// One file's marks, as the merge collects them.
pub(crate) struct FileMarks {
    /// `a`-`z`, then `"`, `^` and `.`.
    pub(crate) marks: [ShadaEntry; 29],
    /// The change list, oldest first; `changes_size` of them are real.
    pub(crate) changes: [ShadaEntry; JUMPLISTSIZE],
    pub(crate) changes_size: usize,
    /// Marks the old file had for this file under names this Nvim does not
    /// know, kept to be written back.
    pub(crate) additional_marks: Vec<ShadaEntry>,
    pub(crate) greatest_timestamp: Timestamp,
}

impl FileMarks {
    /// One file's marks before any have been collected.
    fn new() -> Box<FileMarks> {
        Box::new(FileMarks {
            marks: [const { ShadaEntry::MISSING }; 29],
            changes: [const { ShadaEntry::MISSING }; JUMPLISTSIZE],
            changes_size: 0,
            additional_marks: Vec::new(),
            greatest_timestamp: 0,
        })
    }
}

/// The buffers a ShaDa write keeps no marks for: those on a removable
/// medium (`'shada'`'s `r` prefixes) and the scratch kinds. Membership only,
/// keyed by address, as the khash set it replaced was.
pub(crate) type RemovableBufs = IdSet<*const Buffer>;

/// Where a global mark's letter lives in `namedfm`: `A`-`Z` first, then the
/// ten numbered marks. `None` for a name that is neither.
fn mark_global_index(name: c_char) -> Option<usize> {
    match name as u8 {
        letter @ b'A'..=b'Z' => Some(usize::from(letter - b'A')),
        digit @ b'0'..=b'9' => Some(NMARKS + usize::from(digit - b'0')),
        _ => None,
    }
}

/// Where a buffer-local mark's name lives in a buffer's mark array: `a`-`z`
/// first, then the three special ones. `None` for a name that is none of
/// them.
fn mark_local_index(name: c_char) -> Option<usize> {
    match name as u8 {
        letter @ b'a'..=b'z' => Some(usize::from(letter - b'a')),
        b'"' => Some(NMARKS),
        b'^' => Some(NMARKS + 1),
        b'.' => Some(NMARKS + 2),
        _ => None,
    }
}

/// Where a mark the file does not place sits: line 1, column 0.
pub(crate) const DEFAULT_POS: Pos = Pos {
    lnum: 1,
    col: 0,
    coladd: 0,
};

/// What a search-pattern entry's fields default to. A pattern is magic and
/// was the last one used unless the file says otherwise. Only `pat` starts
/// out unset: a search-pattern entry that names no pattern is malformed, and
/// `None` is how the parser sees that.
const DEFAULT_SEARCH_PATTERN: KeyDict__shada_search_pat = KeyDict__shada_search_pat {
    magic: Some(true),
    smartcase: Some(false),
    has_line_offset: Some(false),
    place_cursor_at_end: Some(false),
    is_last_used: Some(true),
    is_substitute_pattern: Some(false),
    highlighted: Some(false),
    search_backward: Some(false),
    offset: Some(0),
    pat: None,
};

/// What a register entry defaults to: charwise, unnamed, no width.
const DEFAULT_REGISTER: ShadaRegister = ShadaRegister {
    name: 0,
    motion: MOTION_CHARWISE,
    contents: Vec::new(),
    is_unnamed: false,
    width: 0,
};

/// The name a mark entry of this kind has when the file does not give one:
/// a jump and a change have none, while a global or local mark the file does
/// not name is the `"` one.
const fn default_mark_name(kind: u64) -> c_char {
    match kind {
        kind::GLOBAL_MARK | kind::LOCAL_MARK => b'"' as c_char,
        _ => 0,
    }
}
