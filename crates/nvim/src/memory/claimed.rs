//! Lengths and counts that input claims.
//!
//! A file read from disk and bytes from a peer are input: a length in a
//! header says how much data *should* follow, not how much does. Sizing an
//! allocation by it before the data is there lets a twelve-byte file ask
//! for four gigabytes, and an allocation failure ends the editor. The rule
//! every reader follows:
//!
//! - **Bytes in memory** (a msgpack string, a ShaDa entry already read): a
//!   claimed length is compared against what is left of the input before
//!   anything is reserved; a container's count reserves at most
//!   [`claimed_capacity`].
//! - **Bytes from a stream** (a file read piece by piece): the payload is
//!   read through [`read_claimed`], whose buffer grows only as bytes
//!   arrive, so a short file fails its read having allocated about what it
//!   held.
//! - **Arrays indexed as a whole** (a spell tree): the claimed length is
//!   checked against the bytes the file has left before the allocation,
//!   when each element costs at least a byte of input.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

/// How much [`read_claimed`] reads before its buffer grows again, and
/// [`claimed_capacity`]'s ceiling: large enough that an honest payload
/// takes few steps, small enough to allocate on any claim.
pub(crate) const CLAIM_STEP: usize = 64 * 1024;

/// Reads the `len` bytes input claims follow, through `read_exact`, which
/// fills the buffer it is handed or fails.
///
/// The buffer starts at most [`CLAIM_STEP`] long and doubles as each piece
/// arrives, so a stream that ends early fails having allocated at most
/// twice what it held. The pieces are contiguous: what comes back is the
/// same as one `read_exact` into a `len`-byte buffer.
pub(crate) fn read_claimed<E>(
    len: usize,
    mut read_exact: impl FnMut(&mut [u8]) -> Result<(), E>,
) -> Result<Vec<u8>, E> {
    let mut out = Vec::with_capacity(len.min(CLAIM_STEP));
    while out.len() < len {
        let start = out.len();
        let step = (len - start).min(start.max(CLAIM_STEP));
        out.resize(start + step, 0);
        read_exact(&mut out[start..])?;
    }
    Ok(out)
}

/// The capacity to reserve for `count` items input claims, when what is
/// left of the input is `remaining` bytes and each item takes at least
/// `min_item_bytes` of them: never more items than the input could hold,
/// and never more than [`CLAIM_STEP`] up front. A vector reserved this way
/// still grows to `count` if the items do arrive.
pub(crate) fn claimed_capacity(count: usize, remaining: usize, min_item_bytes: usize) -> usize {
    count.min(remaining / min_item_bytes.max(1)).min(CLAIM_STEP)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stream over `data` that fails, as `read_exact` does, when asked
    /// for more than is left, and remembers the largest piece asked for.
    struct Stream<'a> {
        data: &'a [u8],
        at: usize,
        widest: usize,
    }

    impl<'a> Stream<'a> {
        fn new(data: &'a [u8]) -> Self {
            Stream {
                data,
                at: 0,
                widest: 0,
            }
        }

        fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), ()> {
            self.widest = self.widest.max(buf.len());
            let end = self.at + buf.len();
            buf.copy_from_slice(self.data.get(self.at..end).ok_or(())?);
            self.at = end;
            Ok(())
        }
    }

    #[test]
    fn a_payload_that_is_there_reads_whole() {
        let data: Vec<u8> = (0..=255u8).cycle().take(3 * CLAIM_STEP + 17).collect();
        let mut stream = Stream::new(&data);
        assert_eq!(
            read_claimed(data.len(), |b| stream.read_exact(b)),
            Ok(data.clone())
        );
        let mut stream = Stream::new(&data);
        assert_eq!(
            read_claimed(5, |b| stream.read_exact(b)),
            Ok(data[..5].to_vec())
        );
        let mut stream = Stream::new(&[]);
        assert_eq!(read_claimed(0, |b| stream.read_exact(b)), Ok(Vec::new()));
    }

    #[test]
    fn a_claim_past_the_input_fails_having_asked_for_little() {
        let mut stream = Stream::new(&[7u8; 100]);
        assert_eq!(
            read_claimed(usize::MAX / 2, |b| stream.read_exact(b)),
            Err(())
        );
        assert!(
            stream.widest <= CLAIM_STEP,
            "asked for {} bytes",
            stream.widest
        );
    }

    #[test]
    fn a_count_reserves_no_more_than_the_input_holds() {
        assert_eq!(claimed_capacity(10, 1000, 1), 10);
        assert_eq!(claimed_capacity(usize::MAX, 12, 1), 12);
        assert_eq!(claimed_capacity(usize::MAX, 12, 4), 3);
        assert_eq!(claimed_capacity(usize::MAX, usize::MAX, 0), CLAIM_STEP);
    }
}
