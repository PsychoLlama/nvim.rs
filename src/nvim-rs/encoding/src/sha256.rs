//! SHA-256 hashing
//!
//! FIPS-180-2 compliant SHA-256 implementation compatible with nvim's sha256.c.

use std::ffi::c_char;
use std::ptr;

/// SHA-256 digest size in bytes
pub const SHA256_SUM_SIZE: usize = 32;

/// SHA-256 block size in bytes
pub const SHA256_BUFFER_SIZE: usize = 64;

/// SHA-256 context structure - matches C layout
#[repr(C)]
pub struct Sha256Context {
    total: [u32; 2],
    state: [u32; 8],
    buffer: [u8; SHA256_BUFFER_SIZE],
}

impl Default for Sha256Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha256Context {
    /// Create a new SHA-256 context
    pub fn new() -> Self {
        Self {
            total: [0, 0],
            state: [
                0x6A09_E667,
                0xBB67_AE85,
                0x3C6E_F372,
                0xA54F_F53A,
                0x510E_527F,
                0x9B05_688C,
                0x1F83_D9AB,
                0x5BE0_CD19,
            ],
            buffer: [0; SHA256_BUFFER_SIZE],
        }
    }

    /// Update the context with more data
    pub fn update(&mut self, input: &[u8]) {
        if input.is_empty() {
            return;
        }

        let mut left = (self.total[0] & (SHA256_BUFFER_SIZE as u32 - 1)) as usize;

        self.total[0] = self.total[0].wrapping_add(input.len() as u32);
        if self.total[0] < input.len() as u32 {
            self.total[1] = self.total[1].wrapping_add(1);
        }

        let mut offset = 0;
        let fill = SHA256_BUFFER_SIZE - left;

        if left > 0 && input.len() >= fill {
            self.buffer[left..left + fill].copy_from_slice(&input[..fill]);
            self.process_block();
            offset += fill;
            left = 0;
        }

        while offset + SHA256_BUFFER_SIZE <= input.len() {
            self.buffer
                .copy_from_slice(&input[offset..offset + SHA256_BUFFER_SIZE]);
            self.process_block();
            offset += SHA256_BUFFER_SIZE;
        }

        if offset < input.len() {
            let remaining = input.len() - offset;
            self.buffer[left..left + remaining].copy_from_slice(&input[offset..]);
        }
    }

    /// Finalize and return the digest
    pub fn finish(mut self) -> [u8; SHA256_SUM_SIZE] {
        let high = (self.total[0] >> 29) | (self.total[1] << 3);
        let low = self.total[0] << 3;

        let mut msglen = [0u8; 8];
        put_u32_be(high, &mut msglen[0..4]);
        put_u32_be(low, &mut msglen[4..8]);

        let last = (self.total[0] & 0x3F) as usize;
        let padn = if last < 56 { 56 - last } else { 120 - last };

        let mut padding = [0u8; 64];
        padding[0] = 0x80;
        self.update(&padding[..padn]);
        self.update(&msglen);

        let mut digest = [0u8; SHA256_SUM_SIZE];
        for (i, chunk) in digest.chunks_mut(4).enumerate() {
            put_u32_be(self.state[i], chunk);
        }

        digest
    }

    fn process_block(&mut self) {
        let mut w = [0u32; 64];

        // Load first 16 words
        for (i, chunk) in self.buffer.chunks(4).enumerate() {
            w[i] = get_u32_be(chunk);
        }

        // Extend to 64 words
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];
        let mut f = self.state[5];
        let mut g = self.state[6];
        let mut h = self.state[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}

/// Round constants
const K: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

#[inline]
fn get_u32_be(b: &[u8]) -> u32 {
    (u32::from(b[0]) << 24) | (u32::from(b[1]) << 16) | (u32::from(b[2]) << 8) | u32::from(b[3])
}

#[inline]
fn put_u32_be(n: u32, b: &mut [u8]) {
    b[0] = (n >> 24) as u8;
    b[1] = (n >> 16) as u8;
    b[2] = (n >> 8) as u8;
    b[3] = n as u8;
}

/// Compute SHA-256 hash of data
pub fn sha256(data: &[u8]) -> [u8; SHA256_SUM_SIZE] {
    let mut ctx = Sha256Context::new();
    ctx.update(data);
    ctx.finish()
}

/// Convert a hash to a hex string
pub fn to_hex(hash: &[u8; SHA256_SUM_SIZE]) -> String {
    let mut s = String::with_capacity(SHA256_SUM_SIZE * 2);
    for byte in hash {
        s.push_str(&format!("{:02x}", byte));
    }
    s
}

// FFI functions

/// Initialize a SHA-256 context.
///
/// # Safety
///
/// `ctx` must be a valid pointer to a `Sha256Context`.
#[no_mangle]
pub unsafe extern "C" fn rs_sha256_start(ctx: *mut Sha256Context) {
    if !ctx.is_null() {
        unsafe { *ctx = Sha256Context::new() };
    }
}

/// Update a SHA-256 context with more data.
///
/// # Safety
///
/// `ctx` must be a valid pointer to a `Sha256Context`.
/// `input` must be a valid pointer to at least `length` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_sha256_update(
    ctx: *mut Sha256Context,
    input: *const u8,
    length: usize,
) {
    if ctx.is_null() || (input.is_null() && length > 0) {
        return;
    }

    if length == 0 {
        return;
    }

    let input_slice = unsafe { std::slice::from_raw_parts(input, length) };
    unsafe { (*ctx).update(input_slice) };
}

/// Finalize a SHA-256 context and get the digest.
///
/// # Safety
///
/// `ctx` must be a valid pointer to a `Sha256Context`.
/// `digest` must be a valid pointer to at least 32 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_sha256_finish(ctx: *mut Sha256Context, digest: *mut u8) {
    if ctx.is_null() || digest.is_null() {
        return;
    }

    // Create a copy since finish consumes the context
    let ctx_copy = Sha256Context {
        total: unsafe { (*ctx).total },
        state: unsafe { (*ctx).state },
        buffer: unsafe { (*ctx).buffer },
    };

    let result = ctx_copy.finish();
    unsafe {
        ptr::copy_nonoverlapping(result.as_ptr(), digest, SHA256_SUM_SIZE);
    }
}

/// Static buffer for sha256_bytes result
static mut SHA256_HEX_BUFFER: [u8; SHA256_BUFFER_SIZE + 1] = [0; SHA256_BUFFER_SIZE + 1];

/// Compute SHA-256 hash of data and return hex string.
///
/// Returns a pointer to a static buffer containing the hex digest.
/// If `salt` is not null, it is also included in the hash.
///
/// # Safety
///
/// `buf` must be a valid pointer to at least `buf_len` bytes.
/// `salt` must be null or a valid pointer to at least `salt_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_sha256_bytes(
    buf: *const u8,
    buf_len: usize,
    salt: *const u8,
    salt_len: usize,
) -> *const c_char {
    if buf.is_null() && buf_len > 0 {
        return ptr::null();
    }

    let mut ctx = Sha256Context::new();

    if buf_len > 0 {
        let buf_slice = unsafe { std::slice::from_raw_parts(buf, buf_len) };
        ctx.update(buf_slice);
    }

    if !salt.is_null() && salt_len > 0 {
        let salt_slice = unsafe { std::slice::from_raw_parts(salt, salt_len) };
        ctx.update(salt_slice);
    }

    let digest = ctx.finish();

    // Write hex to static buffer
    unsafe {
        for (i, byte) in digest.iter().enumerate() {
            let hex = format!("{:02x}", byte);
            SHA256_HEX_BUFFER[i * 2] = hex.as_bytes()[0];
            SHA256_HEX_BUFFER[i * 2 + 1] = hex.as_bytes()[1];
        }
        SHA256_HEX_BUFFER[64] = 0;
    }

    unsafe { SHA256_HEX_BUFFER.as_ptr().cast() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_empty() {
        let hash = sha256(b"");
        let hex = to_hex(&hash);
        assert_eq!(
            hex,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_abc() {
        let hash = sha256(b"abc");
        let hex = to_hex(&hash);
        assert_eq!(
            hex,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_sha256_long() {
        let hash = sha256(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");
        let hex = to_hex(&hash);
        assert_eq!(
            hex,
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn test_sha256_million_a() {
        // 1 million 'a' characters
        let mut ctx = Sha256Context::new();
        let block = [b'a'; 1000];
        for _ in 0..1000 {
            ctx.update(&block);
        }
        let hash = ctx.finish();
        let hex = to_hex(&hash);
        assert_eq!(
            hex,
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }
}
