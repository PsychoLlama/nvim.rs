//! Base64 encoding and decoding (RFC 4648).
//!
//! Pure functions over byte slices: no globals, callable from any thread
//! (`vim.base64` is exposed to `vim.uv.new_thread` threads).

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Index+1 of each alphabet byte; 0 marks bytes outside the alphabet.
const CHAR_TO_INDEX: [u8; 256] = {
    let mut table = [0u8; 256];
    let mut i = 0;
    while i < ALPHABET.len() {
        table[ALPHABET[i] as usize] = i as u8 + 1;
        i += 1;
    }
    table
};

pub fn encode(src: &[u8]) -> String {
    let mut dest = Vec::with_capacity((src.len() + 2) / 3 * 4);
    let mut chunks = src.chunks_exact(3);
    for chunk in &mut chunks {
        let bits = (chunk[0] as u32) << 16 | (chunk[1] as u32) << 8 | chunk[2] as u32;
        dest.push(ALPHABET[(bits >> 18) as usize]);
        dest.push(ALPHABET[(bits >> 12 & 0x3f) as usize]);
        dest.push(ALPHABET[(bits >> 6 & 0x3f) as usize]);
        dest.push(ALPHABET[(bits & 0x3f) as usize]);
    }
    match *chunks.remainder() {
        [a] => {
            dest.push(ALPHABET[(a >> 2) as usize]);
            dest.push(ALPHABET[((a & 0x3) << 4) as usize]);
            dest.extend_from_slice(b"==");
        }
        [a, b] => {
            dest.push(ALPHABET[(a >> 2) as usize]);
            dest.push(ALPHABET[((a & 0x3) << 4 | b >> 4) as usize]);
            dest.push(ALPHABET[((b & 0xf) << 2) as usize]);
            dest.push(b'=');
        }
        _ => {}
    }
    String::from_utf8(dest).expect("base64 output is ASCII")
}

/// Strict decode: requires canonical padding and zeroed leftover bits,
/// mirroring the upstream C validation exactly. `None` means invalid input.
pub fn decode(src: &[u8]) -> Option<Vec<u8>> {
    if src.len() % 4 != 0 {
        return None;
    }
    let mut dest = Vec::with_capacity(src.len() / 4 * 3);
    let mut acc: u32 = 0;
    let mut acc_len: u32 = 0;
    let mut leftover = None;
    for (i, &c) in src.iter().enumerate() {
        let d = CHAR_TO_INDEX[c as usize];
        if d == 0 {
            if c != b'=' {
                return None;
            }
            leftover = Some(i);
            break;
        }
        acc = (acc << 6 & 0xfff) + (d as u32 - 1);
        acc_len += 6;
        if acc_len >= 8 {
            acc_len -= 8;
            dest.push((acc >> acc_len) as u8);
        }
    }
    if acc_len > 4 || acc & ((1 << acc_len) - 1) != 0 {
        return None;
    }
    if let Some(i) = leftover {
        let padding = &src[i..];
        if padding.iter().any(|&c| c != b'=') || padding.len() as u32 != acc_len / 2 {
            return None;
        }
    }
    Some(dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc4648_vectors() {
        for (plain, encoded) in [
            (&b""[..], ""),
            (b"f", "Zg=="),
            (b"fo", "Zm8="),
            (b"foo", "Zm9v"),
            (b"foob", "Zm9vYg=="),
            (b"fooba", "Zm9vYmE="),
            (b"foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(encode(plain), encoded);
            assert_eq!(decode(encoded.as_bytes()).as_deref(), Some(plain));
        }
    }

    #[test]
    fn encode_covers_all_chunk_paths() {
        let src: Vec<u8> = (0..=255).collect();
        assert_eq!(decode(encode(&src).as_bytes()).as_deref(), Some(&src[..]));
    }

    #[test]
    fn decode_rejects_invalid() {
        for bad in [
            "Zg=",      // truncated padding
            "Zg",       // length not a multiple of 4
            "Z===",     // over-padded
            "====",     // padding only
            "Zm=8",     // data after padding
            "Zh==",     // nonzero leftover bits
            "Zm9v!A==", // byte outside the alphabet
        ] {
            assert_eq!(decode(bad.as_bytes()), None, "{bad:?}");
        }
    }
}
