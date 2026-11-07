//! What the memline refuses: a swap file that cannot be found, opened or
//! believed while recovering, a `:preserve` with nothing to preserve into,
//! and a file name that is a loop of links -- and, apart, a line that could
//! not be appended or deleted ([`LineError`]).
//!
//! The code below `ml_recover`, `ml_preserve` and the swap and undo file
//! namers answers a [`MemlineError`]; each of those reports it, once, with
//! [`MemlineError::report`]. The texts are upstream's, byte for byte.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::highlight_group::HLF_E;
use crate::memory::ThinCString;
use crate::message::{msg_display, msg_end, msg_start, msg_str_hl};
use crate::message_fmt::{emsg_text, msg_cstr};
use crate::tr;
use crate::types::Failed;

use super::note;

/// Why the memline could not do what it was asked.
///
/// The block-zero refusals are not errors in Vim's sense: they are a report
/// built from pieces in the error highlight, which [`report`](Self::report)
/// puts together; their `Display` is the same text in one piece.
#[derive(Debug, thiserror::Error)]
pub(crate) enum MemlineError {
    /// `:recover` found no swap file for the name.
    #[error("{}", tr!("E305: No swap file found for {}", msg_cstr(.0.as_cstr())))]
    NoSwapFile(ThinCString),
    /// The swap file to recover from would not open.
    #[error("{}", tr!("E306: Cannot open {}", msg_cstr(.0.as_cstr())))]
    CannotOpen(ThinCString),
    /// Block zero could not be read at all.
    #[error("Unable to read block 0 from {}", msg_cstr(.0.as_cstr()))]
    NoBlockZero(ThinCString),
    /// Block zero was written by Vim 3.0.
    #[error("{} cannot be used with this version of Nvim.", msg_cstr(.0.as_cstr()))]
    Vim3(ThinCString),
    /// Block zero does not start with a swap file's id.
    #[error(
        "{}",
        tr!("E307: {} does not look like a Nvim swap file", msg_cstr(.0.as_cstr()))
    )]
    NotSwapFile(ThinCString),
    /// Block zero's magic numbers were written by another build, or on
    /// another machine: the swap file and the host name it records (boxed:
    /// two names would widen every memline `Result`).
    #[error("{} cannot be used on this computer.", msg_cstr(.0[0].as_cstr()))]
    OtherComputer(Box<[ThinCString; 2]>),
    /// Block zero claims a page smaller than a swap file can have.
    #[error("{} has been damaged (page size is smaller than minimum value).", msg_cstr(.0.as_cstr()))]
    PageTooSmall(ThinCString),
    /// The first data block could not be read.
    #[error("{}", tr!("E309: Unable to read block 1 from {}", msg_cstr(.0.as_cstr())))]
    NoBlockOne(ThinCString),
    /// The first block is not a data block.
    #[error(
        "{}",
        tr!("E310: Block 1 ID wrong ({} not a .swp file?)", msg_cstr(.0.as_cstr()))
    )]
    BlockOneId(ThinCString),
    /// `:preserve` without a swap file.
    #[error("{}", tr!("E313: Cannot preserve, there is no swap file"))]
    NothingToPreserve,
    /// A sync or a block lookup failed while preserving.
    #[error("{}", tr!("E314: Preserve failed"))]
    PreserveFailed,
    /// Resolving the name's links went round a hundred times.
    #[error("{}", tr!("E773: Symlink loop for \"{}\"", msg_cstr(.0.as_cstr())))]
    SymlinkLoop(ThinCString),
    /// The user chose no swap file from the list: nothing to say.
    #[error("no swap file chosen")]
    NoneChosen,
    /// What the code this calls reported itself.
    #[error(transparent)]
    Reported(#[from] Failed),
}

// Two words, so a `Result` carrying one is no wider than it has to be.
const _: () = assert!(size_of::<MemlineError>() <= 16);

impl MemlineError {
    /// Puts the message out, unless it is already out or there is none.
    pub(crate) fn report(&self) {
        let hl_id = HLF_E;
        match self {
            Self::NoneChosen | Self::Reported(_) => {}
            Self::NoBlockZero(swap) => {
                msg_start();
                note(c"Unable to read block 0 from ", hl_id);
                msg_display(swap.as_cstr(), hl_id, true);
                note(
                    c"\nMaybe no changes were made or Nvim did not update the swap file.",
                    hl_id,
                );
                msg_end();
            }
            Self::Vim3(swap) => {
                msg_start();
                msg_display(swap.as_cstr(), 0, true);
                note(c" cannot be used with this version of Nvim.\n", 0);
                note(c"Use Vim version 3.0.\n", 0);
                msg_end();
            }
            Self::OtherComputer(names) => {
                let [swap, host] = &**names;
                msg_start();
                msg_display(swap.as_cstr(), hl_id, true);
                note(c" cannot be used on this computer.\n", hl_id);
                note(c"The file was created on ", hl_id);
                msg_str_hl(host.as_cstr(), hl_id, true);
                note(c",\nor the file has been damaged.", hl_id);
                msg_end();
            }
            Self::PageTooSmall(swap) => {
                msg_start();
                msg_display(swap.as_cstr(), hl_id, true);
                note(
                    c" has been damaged (page size is smaller than minimum value).\n",
                    hl_id,
                );
                msg_end();
            }
            _ => {
                emsg_text(self.to_string());
            }
        }
    }
}

/// Why a line could not be appended to, or deleted from, a memline.
///
/// Apart from [`MemlineError`] because appending and deleting are the
/// editor's hottest memline calls: this is one byte, `Copy`, with no drop
/// glue for the file read's per-line `Result` to carry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineError {
    /// The line number is not in the buffer, or the buffer has no memline:
    /// nothing is said, as upstream's `FAIL` there says nothing.
    NoSuchLine,
    /// The memline failed underneath: the memfile would not open (said by
    /// the open), or a block could not be read, found, split or freed (an
    /// internal error is said where damage is detected; a block the memfile
    /// could not read is not).
    Broken,
}

/// The memfile that would not open has said why.
impl From<Failed> for LineError {
    fn from(_: Failed) -> Self {
        Self::Broken
    }
}
