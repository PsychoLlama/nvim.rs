//! FIPS 180-2 test vectors, previously asserted at runtime by the C
//! `sha256_self_test()`.

use c2rust_neovim::src::nvim::sha256::{hex_digest, Sha256, SHA256_SUM_SIZE};

#[test]
fn fips_180_2_vector_1() {
    assert_eq!(
        hex_digest(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn fips_180_2_vector_2() {
    assert_eq!(
        hex_digest(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
}

#[test]
fn fips_180_2_vector_3_streamed() {
    // One million 'a's, fed in 1000-byte chunks like the old self-test.
    let mut ctx = Sha256::new();
    for _ in 0..1000 {
        ctx.update(&[b'a'; 1000]);
    }
    let digest = ctx.finish();
    assert_eq!(digest.len(), SHA256_SUM_SIZE);
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(
        hex,
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

#[test]
fn empty_input() {
    assert_eq!(
        hex_digest(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn unaligned_streaming_matches_one_shot() {
    // Feed the same message in ragged chunk sizes that straddle the 64-byte
    // block boundary; the digest must not depend on the chunking.
    let message: Vec<u8> = (0u16..300).map(|i| (i % 251) as u8).collect();
    let expected = hex_digest(&message);
    for chunk_size in [1, 3, 63, 64, 65, 127] {
        let mut ctx = Sha256::new();
        for chunk in message.chunks(chunk_size) {
            ctx.update(chunk);
        }
        let hex: String = ctx.finish().iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, expected, "chunk_size={chunk_size}");
    }
}
