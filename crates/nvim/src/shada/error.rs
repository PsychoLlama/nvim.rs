//! What reading and writing a ShaDa file refuse.
//!
//! The file layer, the entry reader and the entry parsers answer a
//! [`ShadaError`]; the read loop, the merge loop and the two entry points
//! (`shada_read_file`, `shada_write_file`) report it, once, where they decide
//! what happens next — skip the entry, stop reading, or carry on writing.
//! The texts are upstream's, byte for byte.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};
use std::ffi::CString;

use crate::memory::ThinCString;
use crate::message_fmt::{emsg_text, msg_cstr, msg_cstr_opt};
use crate::os::cshim::gettext;
use crate::os::fs::os_strerror;
use crate::tr;

/// Why a ShaDa file, or one entry in it, could not be read or written.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ShadaError {
    /// ShaDa is off (`-i NONE`, `--clean`): nothing to read or write, and
    /// nothing to say.
    #[error("ShaDa is off")]
    Off,
    /// Refused without a word, as upstream does: no temporary name could
    /// be made from the file's.
    #[error("refused without a message")]
    Quiet,
    /// Closing a file failed, with libuv's code.
    #[error(
        "{}",
        tr!("E886: System error while closing ShaDa file: {}", msg_cstr(os_strerror(*.0)))
    )]
    Close(c_int),
    /// The file to read would not open.
    #[error(
        "{}",
        tr!(
            "E886: System error while opening ShaDa file {} for reading: {}",
            msg_cstr(.0.as_cstr()),
            msg_cstr(os_strerror(*.1))
        )
    )]
    OpenForReading(ThinCString, c_int),
    /// The old file, to merge before writing, would not open.
    #[error(
        "{}",
        tr!(
            "E886: System error while opening ShaDa file {} for reading to merge before writing it: {}",
            msg_cstr(.0.as_cstr()),
            msg_cstr(os_strerror(*.1))
        )
    )]
    OpenForMerging(ThinCString, c_int),
    /// The temporary file to write would not open.
    #[error(
        "{}",
        tr!(
            "E886: System error while opening temporary ShaDa file {} for writing: {}",
            msg_cstr(.0.as_cstr()),
            msg_cstr(os_strerror(*.1))
        )
    )]
    OpenTemp(ThinCString, c_int),
    /// Every temporary name, `.tmp.a` to `.tmp.z`, is taken.
    #[error(
        "{}",
        tr!("E138: All {}.tmp.X files exist, cannot write ShaDa file!", msg_cstr(.0.as_cstr()))
    )]
    AllTempsTaken(ThinCString),
    /// The directory the file lives in could not be made: the directory
    /// that failed (if known), and libuv's word for why.
    #[error(
        "{}",
        tr!(
            "E886: Failed to create directory {} for writing ShaDa file: {}",
            msg_cstr(.0.0.as_ref().map_or(c"[NULL]", ThinCString::as_cstr)),
            msg_cstr(.0.1)
        )
    )]
    CreateDir(Box<(Option<ThinCString>, &'static CStr)>),
    /// The file to write would not open.
    #[error(
        "{}",
        tr!(
            "E886: System error while opening ShaDa file {} for writing: {}",
            msg_cstr(.0.as_cstr()),
            msg_cstr(os_strerror(*.1))
        )
    )]
    OpenForWriting(ThinCString, c_int),
    /// The temporary file was left behind: it, and the file it was for.
    #[error(
        "{}",
        tr!(
            "E136: Do not forget to remove {} or rename it manually to {}.",
            msg_cstr(.0[0].as_cstr()),
            msg_cstr(.0[1].as_cstr())
        )
    )]
    TempLeft(Box<[ThinCString; 2]>),
    /// The old file is not ShaDa, so the new one did not replace it.
    #[error(
        "{}",
        tr!(
            "E136: Did not rename {} because {} does not look like a ShaDa file",
            msg_cstr(.0[0].as_cstr()),
            msg_cstr(.0[1].as_cstr())
        )
    )]
    NotRenamedNotShada(Box<[ThinCString; 2]>),
    /// Writing went wrong, so the new file did not replace the old.
    #[error(
        "{}",
        tr!(
            "E136: Did not rename {} to {} because there were errors during writing it",
            msg_cstr(.0[0].as_cstr()),
            msg_cstr(.0[1].as_cstr())
        )
    )]
    NotRenamedErrors(Box<[ThinCString; 2]>),
    /// The old file may not be replaced by this process.
    #[error("{}", tr!("E137: ShaDa file is not writable: {}", msg_cstr(.0.as_cstr())))]
    NotWritable(ThinCString),
    /// Giving the new file the old one's owner failed, with libuv's code.
    #[error(
        "{}",
        tr!(
            "E136: Failed setting uid and gid for file {}: {}",
            msg_cstr(.0.as_cstr()),
            msg_cstr(os_strerror(*.1))
        )
    )]
    Chown(ThinCString, c_int),
    /// The rename over the old file failed.
    #[error(
        "{}",
        tr!(
            "E136: Can't rename ShaDa file from {} to {}!",
            msg_cstr(.0[0].as_cstr()),
            msg_cstr(.0[1].as_cstr())
        )
    )]
    Rename(Box<[ThinCString; 2]>),
    /// A read failed, with libuv's code.
    #[error(
        "{}",
        tr!("E886: System error while reading ShaDa file: {}", msg_cstr(os_strerror(*.0)))
    )]
    Read(c_int),
    /// The file ended inside an entry that said it was this long.
    #[error(
        "{}",
        tr!(
            "E576: Error while reading ShaDa file: last entry specified that it occupies {} bytes, but file ended earlier",
            .0
        )
    )]
    ReadShort(u64),
    /// Skipping an entry failed, with libuv's code.
    #[error(
        "{}",
        tr!("E886: System error while skipping in ShaDa file: {}", msg_cstr(os_strerror(*.0)))
    )]
    Skip(c_int),
    /// Skipping an entry read fewer bytes than asked, with no error.
    #[error(
        "{}",
        tr!(
            "E886: System error while skipping in ShaDa file: {}",
            msg_cstr(gettext(c"too few bytes read"))
        )
    )]
    SkipShort,
    /// The file ended inside an entry that was being skipped.
    #[error(
        "{}",
        tr!(
            "E576: Reading ShaDa file: last entry specified that it occupies {} bytes, but file ended earlier",
            .0
        )
    )]
    SkipEnded(u64),
    /// Reading one of an entry's header integers failed, with libuv's
    /// code.
    #[error(
        "{}",
        tr!(
            "E886: System error while reading integer from ShaDa file: {}",
            msg_cstr(os_strerror(*.0))
        )
    )]
    ReadInteger(c_int),
    /// The file ended where an entry's header integer belonged.
    #[error(
        "{}",
        tr!(
            "E576: Error while reading ShaDa file: expected positive integer at position {}, but got nothing",
            .0
        )
    )]
    NoInteger(u64),
    /// An entry's header integer is not a positive integer.
    #[error(
        "{}",
        tr!("E576: Error while reading ShaDa file: expected positive integer at position {}", .0)
    )]
    NotInteger(u64),
    /// An entry says it is longer than anything can be.
    #[error(
        "{}",
        tr!(
            "E576: Error while reading ShaDa file: there is an item at position {} that is stated to be too long",
            .0
        )
    )]
    TooLong(u64),
    /// An entry of the internal "missing" kind.
    #[error(
        "{}",
        tr!(
            "E576: Error while reading ShaDa file: there is an item at position {} that must not be there: Missing items are for internal uses only",
            .0
        )
    )]
    MissingKind(u64),
    /// A payload msgpack read whole with bytes left over.
    #[error(
        "{}",
        tr!("E576: Failed to parse ShaDa file: extra bytes in msgpack string at position {}", .0)
    )]
    ExtraBytes(u64),
    /// A payload msgpack ran out in.
    #[error(
        "{}",
        tr!("E576: Failed to parse ShaDa file: incomplete msgpack string at position {}", .0)
    )]
    Incomplete(u64),
    /// A payload that is not msgpack.
    #[error(
        "{}",
        tr!("E576: Failed to parse ShaDa file due to a msgpack parser error at position {}", .0)
    )]
    NotMsgpack(u64),
    /// One entry, at this position, says something it cannot.
    #[error("{}", .0.render(*.1))]
    Entry(EntryFault, u64),
    /// One entry's map, at this position, did not read as its keyset: the
    /// entry, the position, and the keyset reader's reason.
    #[error("{}", render_keyed(&.0))]
    Keyed(Box<(Keyed, u64, Option<CString>)>),
}

// Two words, so a `Result` carrying one is no wider than it has to be.
const _: () = assert!(size_of::<ShadaError>() <= 16);

impl ShadaError {
    /// Puts the message out, unless there is none.
    pub(crate) fn report(&self) {
        if !matches!(self, Self::Off | Self::Quiet) {
            emsg_text(self.to_string());
        }
    }

    /// The two names an `E136` speaks of.
    pub(crate) fn names(first: &CStr, second: &CStr) -> Box<[ThinCString; 2]> {
        Box::new([
            ThinCString::from_cstr(first),
            ThinCString::from_cstr(second),
        ])
    }
}

/// What one entry says that it cannot (`E575`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntryFault {
    ItemExtraBytes,
    PatternWithoutPattern,
    MarkWithN,
    MarkWithoutFile,
    MarkLine,
    MarkColumn,
    RegisterWithoutContents,
    HistoryNotArray,
    HistoryType,
    HistoryString,
    HistoryNul,
    SearchHistoryWithoutSeparator,
    SearchHistorySeparatorType,
    VariableNotArray,
    VariableNameType,
    VariableType,
    VariableValue,
    SubNotArray,
    SubType,
    BufferListNotArray,
    BufferListLine,
    BufferListColumn,
    BufferListWithoutFile,
}

impl EntryFault {
    /// The message, for an entry at `pos`.
    fn render(self, pos: u64) -> String {
        match self {
            Self::ItemExtraBytes => tr!(
                "E575: Error while reading ShaDa file: item entry at position {} additional bytes",
                pos
            ),
            Self::PatternWithoutPattern => tr!(
                "E575: Error while reading ShaDa file: search pattern entry at position {} has no pattern",
                pos
            ),
            Self::MarkWithN => tr!(
                "E575: Error while reading ShaDa file: mark entry at position {} has n key which is only valid for local and global mark entries",
                pos
            ),
            Self::MarkWithoutFile => tr!(
                "E575: Error while reading ShaDa file: mark entry at position {} is missing file name",
                pos
            ),
            Self::MarkLine => tr!(
                "E575: Error while reading ShaDa file: mark entry at position {} has invalid line number",
                pos
            ),
            Self::MarkColumn => tr!(
                "E575: Error while reading ShaDa file: mark entry at position {} has invalid column number",
                pos
            ),
            Self::RegisterWithoutContents => tr!(
                "E575: Error while reading ShaDa file: register entry at position {} has rc key with missing or empty array",
                pos
            ),
            Self::HistoryNotArray => tr!(
                "E575: Error while reading ShaDa file: history entry at position {} is not an array with enough elements",
                pos
            ),
            Self::HistoryType => tr!(
                "E575: Error while reading ShaDa file: history entry at position {} has wrong history type type",
                pos
            ),
            Self::HistoryString => tr!(
                "E575: Error while reading ShaDa file: history entry at position {} has wrong history string type",
                pos
            ),
            Self::HistoryNul => tr!(
                "E575: Error while reading ShaDa file: history entry at position {} contains string with zero byte inside",
                pos
            ),
            Self::SearchHistoryWithoutSeparator => tr!(
                "E575: Error while reading ShaDa file: search history entry at position {} does not have separator character",
                pos
            ),
            Self::SearchHistorySeparatorType => tr!(
                "E575: Error while reading ShaDa file: search history entry at position {} has wrong history separator type",
                pos
            ),
            Self::VariableNotArray => tr!(
                "E575: Error while reading ShaDa file: variable entry at position {} is not an array with enough elements",
                pos
            ),
            Self::VariableNameType => tr!(
                "E575: Error while reading ShaDa file: variable entry at position {} has wrong variable name type",
                pos
            ),
            Self::VariableType => tr!(
                "E575: Error while reading ShaDa file: variable entry at position {} has wrong variable type",
                pos
            ),
            Self::VariableValue => tr!(
                "E575: Error while reading ShaDa file: variable entry at position {} has value that cannot be converted to the Vimscript value",
                pos
            ),
            Self::SubNotArray => tr!(
                "E575: Error while reading ShaDa file: sub string entry at position {} is not an array with enough elements",
                pos
            ),
            Self::SubType => tr!(
                "E575: Error while reading ShaDa file: sub string entry at position {} has wrong sub string type",
                pos
            ),
            Self::BufferListNotArray => tr!(
                "E575: Error while reading ShaDa file: buffer list entry at position {} is not an array",
                pos
            ),
            Self::BufferListLine => tr!(
                "E575: Error while reading ShaDa file: buffer list at position {} contains entry with invalid line number",
                pos
            ),
            Self::BufferListColumn => tr!(
                "E575: Error while reading ShaDa file: buffer list at position {} contains entry with invalid column number",
                pos
            ),
            Self::BufferListWithoutFile => tr!(
                "E575: Error while reading ShaDa file: buffer list at position {} contains entry that does not have a file name",
                pos
            ),
        }
    }
}

/// The entries read as a keyset, for the `E575`s that pass on what the
/// keyset reader said.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Keyed {
    SearchPattern,
    Mark,
    Register,
    BufferList,
}

/// The `E575` for a keyset that did not read: `[NULL]` when the reader gave
/// no reason.
fn render_keyed(failure: &(Keyed, u64, Option<CString>)) -> String {
    let (keyed, pos, why) = failure;
    let why = msg_cstr_opt(why.as_deref());
    match keyed {
        Keyed::SearchPattern => tr!(
            "E575: Error while reading ShaDa file: search pattern entry at position {} {}",
            pos,
            why
        ),
        Keyed::Mark => tr!(
            "E575: Error while reading ShaDa file: mark entry at position {} {}",
            pos,
            why
        ),
        Keyed::Register => tr!(
            "E575: Error while reading ShaDa file: register entry at position {} {}",
            pos,
            why
        ),
        Keyed::BufferList => tr!(
            "E575: Error while reading ShaDa file: buffer list at position {} contains entry that {}",
            pos,
            why
        ),
    }
}
