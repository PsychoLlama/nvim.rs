//! [`Spl`] — the one handle a `.spl` or `.sug` file is read through.
//!
//! Everything on the read side used to go through a `*mut FILE` and libc's
//! `getc`, with the end of the file reported as `-1` and every caller
//! deciding for itself whether that meant "truncated" or "broken". This
//! wraps a buffered [`File`] instead: each read answers
//! `Result<_, SpellReadError>`, and the three variants are exactly the three
//! `SP_*` codes the format's error reporting has always had.
//!
//! # Which error means what
//!
//! - [`SpellReadError::Trunc`] — the file ended where more was expected.
//!   Reported as `E758: Truncated spell file`.
//! - [`SpellReadError::Format`] — the bytes are there but do not make sense.
//!   Reported as `E759: Format error in spell file`.
//! - [`SpellReadError::Other`] — the read itself failed, or a string's
//!   *payload* ran out after its length said otherwise. Not reported at all:
//!   the load stops silently, which is what upstream does.
//!
//! The split between `Trunc` and `Other` is finer than "did the file end":
//! [`Spl::read_cnt_string`] answers `Trunc` when the *count* is short and
//! `Other` when the count was read but the bytes behind it were not. That
//! difference is visible — `test_spellfile.vim`'s format cases assert `E758`
//! on one side of it and no message at all on the other — so the two stay
//! apart.
//!
//! # Byte order
//!
//! Every multi-byte number in a `.spl` is most significant byte first, so
//! [`Spl::get2c`] and friends accumulate rather than transmute. `get4c`
//! wraps into a negative `c_int` when the top bit is set, which is what the
//! callers' `< 0` tests are looking for: a section that claims two gigabytes
//! is treated as a truncated one.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;
use std::fs::File;
use std::io::{BufReader, ErrorKind, Read};
use std::path::Path;

use crate::types::time_t;

/// Why a `.spl` or `.sug` could not be read any further.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(super) enum SpellReadError {
    /// The file ended early.
    #[error("truncated spell file")]
    Trunc,
    /// The bytes are there but do not describe anything.
    #[error("format error in spell file")]
    Format,
    /// The read failed, or a counted string was shorter than its count.
    #[error("failed to read spell file")]
    Other,
}

/// What every reader here answers.
pub(super) type SplResult<T> = Result<T, SpellReadError>;

/// A `.spl` or `.sug` file, open for reading.
///
/// The one byte of pushback exists for `SN_COMPOUND`, which has to look at
/// the byte after the syllable maximum to tell the old section layout from
/// the new one and put it back when it turns out to be a flag.
pub(super) struct Spl {
    file: BufReader<File>,
    /// A byte handed back by [`Spl::unget`], read before the file again.
    pending: Option<u8>,
    /// The last read failure that was not the end of the file, kept so
    /// `E5042` can name it.
    why: Option<std::io::Error>,
}

impl Spl {
    /// Open `path` for reading.
    pub(super) fn open(path: &Path) -> std::io::Result<Self> {
        Ok(Self {
            file: BufReader::new(File::open(path)?),
            pending: None,
            why: None,
        })
    }

    /// What the last failed read said, for `E5042`.
    ///
    /// Upstream passes `ferror`'s answer to `strerror`, which renders every
    /// failure as errno 1 whatever it was; this names the real one.
    pub(super) fn last_error(&self) -> String {
        match &self.why {
            Some(e) => e.to_string(),
            None => String::from("Success"),
        }
    }

    /// The next byte, or `None` at the end of the file — libc's `getc`,
    /// with its error and end-of-file answers still folded together.
    pub(super) fn getc(&mut self) -> Option<u8> {
        if let Some(b) = self.pending.take() {
            return Some(b);
        }
        let mut b = [0u8; 1];
        loop {
            return match self.file.read(&mut b) {
                Ok(0) => None,
                Ok(_) => Some(b[0]),
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => {
                    self.why = Some(e);
                    None
                }
            };
        }
    }

    /// Put one byte back, to be read again next.
    ///
    /// Only one is ever outstanding; a second replaces the first, as
    /// `ungetc` on a stream with a one-byte pushback buffer would.
    pub(super) fn unget(&mut self, b: u8) {
        self.pending = Some(b);
    }

    /// The next byte, or [`SpellReadError::Trunc`].
    pub(super) fn byte(&mut self) -> SplResult<u8> {
        self.getc().ok_or(SpellReadError::Trunc)
    }

    /// The next `N` bytes, or [`SpellReadError::Trunc`].
    fn array<const N: usize>(&mut self) -> SplResult<[u8; N]> {
        let mut out = [0u8; N];
        for slot in &mut out {
            *slot = self.byte()?;
        }
        Ok(out)
    }

    /// Two bytes as a number, MSB first.
    pub(super) fn get2c(&mut self) -> SplResult<c_int> {
        let [hi, lo] = self.array::<2>()?;
        Ok(c_int::from(u16::from_be_bytes([hi, lo])))
    }

    /// Three bytes as a number, MSB first.
    pub(super) fn get3c(&mut self) -> SplResult<c_int> {
        let [hi, mid, lo] = self.array::<3>()?;
        Ok((c_int::from(hi) << 16) | c_int::from(u16::from_be_bytes([mid, lo])))
    }

    /// Four bytes as a number, MSB first.
    ///
    /// The top bit lands in the sign, which is what the callers' `< 0` tests
    /// are for: a section or a tree claiming two gigabytes is refused as a
    /// truncated one rather than allocated.
    pub(super) fn get4c(&mut self) -> SplResult<c_int> {
        Ok(c_int::from_be_bytes(self.array::<4>()?))
    }

    /// Eight bytes as a timestamp, MSB first.
    pub(super) fn get8ctime(&mut self) -> SplResult<time_t> {
        Ok(time_t::from_be_bytes(self.array::<8>()?))
    }

    /// Fill `buf` exactly, or say why not.
    ///
    /// A short read is [`SpellReadError::Trunc`]; anything else about the
    /// file being unreadable is [`SpellReadError::Other`].
    pub(super) fn read_exact(&mut self, buf: &mut [u8]) -> SplResult<()> {
        let mut from = 0;
        if let Some(b) = self.pending.take() {
            match buf.first_mut() {
                Some(first) => {
                    *first = b;
                    from = 1;
                }
                None => self.pending = Some(b),
            }
        }
        match self.file.read_exact(&mut buf[from..]) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => Err(SpellReadError::Trunc),
            Err(e) => {
                self.why = Some(e);
                Err(SpellReadError::Other)
            }
        }
    }

    /// `n` bytes as a fresh buffer.
    pub(super) fn read_bytes(&mut self, n: usize) -> SplResult<Vec<u8>> {
        let mut buf = vec![0u8; n];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// As [`Spl::read_bytes`], and reject an embedded NUL.
    ///
    /// Several sections hold text that later gets treated as a C string, so
    /// a NUL inside would silently truncate it; the file is malformed
    /// instead.
    pub(super) fn read_nonnul_bytes(&mut self, n: usize) -> SplResult<Vec<u8>> {
        let buf = self.read_bytes(n)?;
        if buf.contains(&0) {
            return Err(SpellReadError::Format);
        }
        Ok(buf)
    }

    /// `n` bytes of string payload.
    ///
    /// Running out here is [`SpellReadError::Other`], not `Trunc`: the file
    /// said how long the string was and then did not have it, which upstream
    /// reports by loading nothing and saying nothing.
    pub(super) fn read_string(&mut self, n: usize) -> SplResult<Vec<u8>> {
        let mut buf = vec![0u8; n];
        for slot in &mut buf {
            *slot = self.getc().ok_or(SpellReadError::Other)?;
        }
        Ok(buf)
    }

    /// A length in `cnt_bytes` bytes, then that many bytes of string.
    ///
    /// The answer is exactly as long as the count said — NULs inside and
    /// all, because a caller that steps over the section by length needs the
    /// declared length, not the C string's. An empty answer means the count
    /// was zero, which several sections treat as "absent".
    pub(super) fn read_cnt_string(&mut self, cnt_bytes: u32) -> SplResult<Vec<u8>> {
        let mut cnt: usize = 0;
        for _ in 0..cnt_bytes {
            cnt = (cnt << 8) | usize::from(self.byte()?);
        }
        self.read_string(cnt)
    }
}

/// The bytes up to the first NUL — what a `.spl` string means once it is
/// handed to something that expects a C string.
pub(super) fn trim_nul(bytes: &[u8]) -> &[u8] {
    match bytes.iter().position(|&b| b == 0) {
        Some(at) => &bytes[..at],
        None => bytes,
    }
}
