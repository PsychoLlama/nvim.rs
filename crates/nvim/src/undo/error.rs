//! What undo refuses: a change to a buffer that may not change, a step to a
//! state the tree does not have, and an undo file that cannot be read,
//! believed or written.
//!
//! The code below the undo commands, `u_savecommon` and the undo-file
//! entries answers an [`UndoError`] or an [`UndoFileError`]; the entry
//! reports it, once. The texts are upstream's, byte for byte.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};

use crate::memory::ThinCString;
use crate::message::{E_MODIFIABLE, E_SANDBOX, E_TEXTLOCK, give_warning, msg};
use crate::message_fmt::{emsg_text, msg_cstr, msg_cstr_opt};
use crate::os::cshim::gettext;
use crate::tr;

use super::verbosely;

/// Why undo will not touch the buffer, or cannot get where it was asked to.
#[derive(Debug, thiserror::Error)]
pub(crate) enum UndoError {
    /// 'modifiable' is off.
    #[error("{}", msg_cstr(gettext(E_MODIFIABLE)))]
    NotModifiable,
    /// In the sandbox.
    #[error("{}", msg_cstr(gettext(E_SANDBOX)))]
    Sandbox,
    /// The text is locked.
    #[error("{}", msg_cstr(gettext(E_TEXTLOCK)))]
    Textlock,
    /// The region to save runs past the end of the buffer.
    #[error("{}", tr!("E881: Line count changed unexpectedly"))]
    LineCountChanged,
    /// `:undo N` named a change the tree does not have.
    #[error("{}", tr!("E830: Undo number {} not found", i64::from(*.0)))]
    NoSuchNumber(c_int),
    /// Nothing older to go to. Not an error: a message.
    #[error("{}", tr!("Already at oldest change"))]
    AtOldest,
    /// Nothing newer to go to. Not an error: a message.
    #[error("{}", tr!("Already at newest change"))]
    AtNewest,
}

const _: () = assert!(size_of::<UndoError>() <= 16);

impl UndoError {
    /// Puts the message out.
    pub(crate) fn report(&self) {
        match self {
            Self::AtOldest => {
                msg(gettext(c"Already at oldest change"), 0);
            }
            Self::AtNewest => {
                msg(gettext(c"Already at newest change"), 0);
            }
            _ => {
                emsg_text(self.to_string());
            }
        }
    }
}

/// Why an undo file was not read or written. The file's name is the
/// entry's, which [`report`](Self::report) is given.
#[derive(Debug)]
pub(crate) enum UndoFileError {
    /// The file to read would not open.
    CannotOpenForReading,
    /// It does not start with the undo file magic.
    NotAnUndoFile,
    /// It is a version this build does not read.
    Incompatible,
    /// It says something a well-formed undo file cannot say: which part.
    Corrupt(Corruption),
    /// It describes text the buffer no longer holds. A warning, said only
    /// under 'verbose' when the read was automatic.
    ContentsChanged,
    /// Refused without a word, as upstream does: a length that is no
    /// length, or a file that ran out mid-record.
    Quiet,
    /// The file to write would not open.
    CannotOpenForWriting,
    /// A write failed part-way.
    WriteFailed,
    /// The last 'undodir' entry could not be made: the directory that
    /// failed, and libuv's word for why.
    CannotCreateDir(Box<(Option<ThinCString>, &'static CStr)>),
}

const _: () = assert!(size_of::<UndoFileError>() <= 16);

impl UndoFileError {
    /// Puts the message about `file_name` out, unless there is none.
    /// `automatic` is a read or write the editor decided on by itself.
    pub(crate) fn report(&self, file_name: &CStr, automatic: bool) {
        // The literals are upstream's msgids, named arguments and all.
        let file_name = msg_cstr(file_name);
        let text = match self {
            Self::Quiet => return,
            Self::ContentsChanged => {
                verbosely(automatic, || {
                    let mesg = gettext(c"File contents changed, cannot use undo info");
                    give_warning(mesg, true, true);
                });
                return;
            }
            Self::CannotOpenForReading => {
                tr!("E822: Cannot open undo file for reading: {file_name}")
            }
            Self::NotAnUndoFile => tr!("E823: Not an undo file: {file_name}"),
            Self::Incompatible => tr!("E824: Incompatible undo file: {file_name}"),
            Self::Corrupt(part) => {
                let mesg = msg_cstr(part.what());
                tr!("E825: Corrupted undo file ({mesg}): {file_name}")
            }
            Self::CannotOpenForWriting => {
                tr!("E828: Cannot open undo file for writing: {file_name}")
            }
            Self::WriteFailed => tr!("E829: Write error in undo file: {file_name}"),
            Self::CannotCreateDir(failure) => {
                let (dir, why) = &**failure;
                let shown = msg_cstr_opt(dir.as_ref().map(ThinCString::as_cstr));
                let why = msg_cstr(why);
                tr!("E5003: Unable to create directory \"{shown}\" for undo file: {why}")
            }
        };
        emsg_text(text);
    }
}

/// The part of an undo file `E825` names, in upstream's words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Corruption {
    UhSeq,
    Truncated,
    EntryEnd,
    ExtmarkTruncated,
    ExtmarkSplice,
    ExtmarkMove,
    ExtmarkType,
    EntrySize,
    LineLength,
    Hash,
    LineLnumCol,
    NumHead,
    EndMarker,
    NumHeadTooSmall,
    DuplicateUhSeq,
}

impl Corruption {
    /// The words `E825` puts in its parentheses. Not translated: upstream
    /// passes them as they are.
    pub(crate) fn what(self) -> &'static CStr {
        match self {
            Self::UhSeq => c"uh_seq",
            Self::Truncated => c"truncated",
            Self::EntryEnd => c"entry end",
            Self::ExtmarkTruncated => c"extmark truncated",
            Self::ExtmarkSplice => c"extmark splice",
            Self::ExtmarkMove => c"extmark move",
            Self::ExtmarkType => c"extmark type",
            Self::EntrySize => c"entry size",
            Self::LineLength => c"line length",
            Self::Hash => c"hash",
            Self::LineLnumCol => c"line lnum/col",
            Self::NumHead => c"num_head",
            Self::EndMarker => c"end marker",
            Self::NumHeadTooSmall => c"num_head too small",
            Self::DuplicateUhSeq => c"duplicate uh_seq",
        }
    }
}
