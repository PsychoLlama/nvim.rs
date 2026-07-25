//! Port of `test/unit/memory_spec.lua`.

use c2rust_neovim::src::nvim::memory::xstrlcat;

use crate::support::cstr;

/// Mirror of the spec's `test_xstrlcat`: seed a `dsize` buffer with `dst`,
/// concatenate `src`, and return the resulting string. Asserts the returned
/// length is the total length xstrlcat tried to create.
fn concat(dst: &str, src: &str, dsize: usize) -> String {
    assert!(dsize >= 1 + dst.len());
    let mut buf = vec![0u8; dsize];
    buf[..dst.len()].copy_from_slice(dst.as_bytes());
    let src = cstr(src);
    let total = unsafe { xstrlcat(buf.as_mut_ptr().cast(), src.as_ptr(), dsize) };
    assert_eq!(dst.len() + src.as_bytes().len(), total);
    let nul = buf.iter().position(|&b| b == 0).unwrap();
    String::from_utf8(buf[..nul].to_vec()).unwrap()
}

/// Mirror of `test_xstrlcat_overlap`: `src` points into `dst` at `src_idx`.
fn concat_overlap(dst: &str, src_idx: usize, dsize: usize) -> String {
    assert!(dsize >= 1 + dst.len());
    let mut buf = vec![0u8; dsize];
    buf[..dst.len()].copy_from_slice(dst.as_bytes());
    let ptr = buf.as_mut_ptr();
    let total = unsafe { xstrlcat(ptr.cast(), ptr.add(src_idx).cast_const().cast(), dsize) };
    assert_eq!(dst.len() + dst.len() - src_idx, total);
    let nul = buf.iter().position(|&b| b == 0).unwrap();
    String::from_utf8(buf[..nul].to_vec()).unwrap()
}

#[test]
fn xstrlcat_concatenates_strings() {
    assert_eq!("ab", concat("a", "b", 3));
    assert_eq!("ab", concat("a", "b", 4096));
    assert_eq!("ABCיהZdefgiיהZ", concat("ABCיהZ", "defgiיהZ", 4096));
    assert_eq!("b", concat("", "b", 4096));
    assert_eq!("a", concat("a", "", 4096));
}

#[test]
fn xstrlcat_concatenates_overlapping_strings() {
    assert_eq!("abcabc", concat_overlap("abc", 0, 7));
    assert_eq!("abca", concat_overlap("abc", 0, 5));
    assert_eq!("abcb", concat_overlap("abc", 1, 5));
    assert_eq!("abcc", concat_overlap("abc", 2, 10));
    assert_eq!("abcabc", concat_overlap("abc", 0, 2343));
}

#[test]
fn xstrlcat_truncates_if_dsize_is_too_small() {
    assert_eq!("a", concat("a", "b", 2));
    assert_eq!("", concat("", "b", 1));
    assert_eq!("ABCיהZd", concat("ABCיהZ", "defgiיהZ", 10));
}
