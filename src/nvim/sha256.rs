use crate::src::nvim::global_cell::GlobalCell;
pub use crate::src::nvim::types::{context_sha256_T, size_t, uint32_t, uint8_t};
extern "C" {
    fn snprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strcpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const SHA256_BUFFER_SIZE: ::core::ffi::c_int = 64 as ::core::ffi::c_int;
pub const SHA256_SUM_SIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn sha256_start(mut ctx: *mut context_sha256_T) {
    (*ctx).total[0 as ::core::ffi::c_int as usize] = 0 as uint32_t;
    (*ctx).total[1 as ::core::ffi::c_int as usize] = 0 as uint32_t;
    (*ctx).state[0 as ::core::ffi::c_int as usize] = 0x6a09e667 as uint32_t;
    (*ctx).state[1 as ::core::ffi::c_int as usize] = 0xbb67ae85 as ::core::ffi::c_uint as uint32_t;
    (*ctx).state[2 as ::core::ffi::c_int as usize] = 0x3c6ef372 as uint32_t;
    (*ctx).state[3 as ::core::ffi::c_int as usize] = 0xa54ff53a as ::core::ffi::c_uint as uint32_t;
    (*ctx).state[4 as ::core::ffi::c_int as usize] = 0x510e527f as uint32_t;
    (*ctx).state[5 as ::core::ffi::c_int as usize] = 0x9b05688c as ::core::ffi::c_uint as uint32_t;
    (*ctx).state[6 as ::core::ffi::c_int as usize] = 0x1f83d9ab as uint32_t;
    (*ctx).state[7 as ::core::ffi::c_int as usize] = 0x5be0cd19 as uint32_t;
}
unsafe extern "C" fn sha256_process(mut ctx: *mut context_sha256_T, mut data: *const uint8_t) {
    let mut temp1: uint32_t = 0;
    let mut temp2: uint32_t = 0;
    let mut W: [uint32_t; 64] = [0; 64];
    let mut A: uint32_t = 0;
    let mut B: uint32_t = 0;
    let mut C: uint32_t = 0;
    let mut D: uint32_t = 0;
    let mut E: uint32_t = 0;
    let mut F: uint32_t = 0;
    let mut G: uint32_t = 0;
    let mut H: uint32_t = 0;
    W[0 as ::core::ffi::c_int as usize] = (*data.offset(0 as ::core::ffi::c_int as isize)
        as uint32_t)
        << 24 as ::core::ffi::c_int
        | (*data.offset((0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as uint32_t)
            << 16 as ::core::ffi::c_int
        | (*data.offset((0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) as uint32_t)
            << 8 as ::core::ffi::c_int
        | *data.offset((0 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as uint32_t;
    W[1 as ::core::ffi::c_int as usize] = (*data.offset(4 as ::core::ffi::c_int as isize)
        as uint32_t)
        << 24 as ::core::ffi::c_int
        | (*data.offset((4 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as uint32_t)
            << 16 as ::core::ffi::c_int
        | (*data.offset((4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) as uint32_t)
            << 8 as ::core::ffi::c_int
        | *data.offset((4 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as uint32_t;
    W[2 as ::core::ffi::c_int as usize] = (*data.offset(8 as ::core::ffi::c_int as isize)
        as uint32_t)
        << 24 as ::core::ffi::c_int
        | (*data.offset((8 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as uint32_t)
            << 16 as ::core::ffi::c_int
        | (*data.offset((8 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) as uint32_t)
            << 8 as ::core::ffi::c_int
        | *data.offset((8 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as uint32_t;
    W[3 as ::core::ffi::c_int as usize] = (*data.offset(12 as ::core::ffi::c_int as isize)
        as uint32_t)
        << 24 as ::core::ffi::c_int
        | (*data.offset((12 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as uint32_t)
            << 16 as ::core::ffi::c_int
        | (*data.offset((12 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) as uint32_t)
            << 8 as ::core::ffi::c_int
        | *data.offset((12 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as uint32_t;
    W[4 as ::core::ffi::c_int as usize] = (*data.offset(16 as ::core::ffi::c_int as isize)
        as uint32_t)
        << 24 as ::core::ffi::c_int
        | (*data.offset((16 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as uint32_t)
            << 16 as ::core::ffi::c_int
        | (*data.offset((16 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) as uint32_t)
            << 8 as ::core::ffi::c_int
        | *data.offset((16 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as uint32_t;
    W[5 as ::core::ffi::c_int as usize] = (*data.offset(20 as ::core::ffi::c_int as isize)
        as uint32_t)
        << 24 as ::core::ffi::c_int
        | (*data.offset((20 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as uint32_t)
            << 16 as ::core::ffi::c_int
        | (*data.offset((20 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) as uint32_t)
            << 8 as ::core::ffi::c_int
        | *data.offset((20 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as uint32_t;
    W[6 as ::core::ffi::c_int as usize] = (*data.offset(24 as ::core::ffi::c_int as isize)
        as uint32_t)
        << 24 as ::core::ffi::c_int
        | (*data.offset((24 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as uint32_t)
            << 16 as ::core::ffi::c_int
        | (*data.offset((24 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) as uint32_t)
            << 8 as ::core::ffi::c_int
        | *data.offset((24 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as uint32_t;
    W[7 as ::core::ffi::c_int as usize] = (*data.offset(28 as ::core::ffi::c_int as isize)
        as uint32_t)
        << 24 as ::core::ffi::c_int
        | (*data.offset((28 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as uint32_t)
            << 16 as ::core::ffi::c_int
        | (*data.offset((28 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) as uint32_t)
            << 8 as ::core::ffi::c_int
        | *data.offset((28 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as uint32_t;
    W[8 as ::core::ffi::c_int as usize] = (*data.offset(32 as ::core::ffi::c_int as isize)
        as uint32_t)
        << 24 as ::core::ffi::c_int
        | (*data.offset((32 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as uint32_t)
            << 16 as ::core::ffi::c_int
        | (*data.offset((32 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) as uint32_t)
            << 8 as ::core::ffi::c_int
        | *data.offset((32 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as uint32_t;
    W[9 as ::core::ffi::c_int as usize] = (*data.offset(36 as ::core::ffi::c_int as isize)
        as uint32_t)
        << 24 as ::core::ffi::c_int
        | (*data.offset((36 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as uint32_t)
            << 16 as ::core::ffi::c_int
        | (*data.offset((36 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) as uint32_t)
            << 8 as ::core::ffi::c_int
        | *data.offset((36 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as uint32_t;
    W[10 as ::core::ffi::c_int as usize] = (*data.offset(40 as ::core::ffi::c_int as isize)
        as uint32_t)
        << 24 as ::core::ffi::c_int
        | (*data.offset((40 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as uint32_t)
            << 16 as ::core::ffi::c_int
        | (*data.offset((40 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) as uint32_t)
            << 8 as ::core::ffi::c_int
        | *data.offset((40 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as uint32_t;
    W[11 as ::core::ffi::c_int as usize] = (*data.offset(44 as ::core::ffi::c_int as isize)
        as uint32_t)
        << 24 as ::core::ffi::c_int
        | (*data.offset((44 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as uint32_t)
            << 16 as ::core::ffi::c_int
        | (*data.offset((44 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) as uint32_t)
            << 8 as ::core::ffi::c_int
        | *data.offset((44 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as uint32_t;
    W[12 as ::core::ffi::c_int as usize] = (*data.offset(48 as ::core::ffi::c_int as isize)
        as uint32_t)
        << 24 as ::core::ffi::c_int
        | (*data.offset((48 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as uint32_t)
            << 16 as ::core::ffi::c_int
        | (*data.offset((48 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) as uint32_t)
            << 8 as ::core::ffi::c_int
        | *data.offset((48 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as uint32_t;
    W[13 as ::core::ffi::c_int as usize] = (*data.offset(52 as ::core::ffi::c_int as isize)
        as uint32_t)
        << 24 as ::core::ffi::c_int
        | (*data.offset((52 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as uint32_t)
            << 16 as ::core::ffi::c_int
        | (*data.offset((52 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) as uint32_t)
            << 8 as ::core::ffi::c_int
        | *data.offset((52 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as uint32_t;
    W[14 as ::core::ffi::c_int as usize] = (*data.offset(56 as ::core::ffi::c_int as isize)
        as uint32_t)
        << 24 as ::core::ffi::c_int
        | (*data.offset((56 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as uint32_t)
            << 16 as ::core::ffi::c_int
        | (*data.offset((56 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) as uint32_t)
            << 8 as ::core::ffi::c_int
        | *data.offset((56 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as uint32_t;
    W[15 as ::core::ffi::c_int as usize] = (*data.offset(60 as ::core::ffi::c_int as isize)
        as uint32_t)
        << 24 as ::core::ffi::c_int
        | (*data.offset((60 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) as uint32_t)
            << 16 as ::core::ffi::c_int
        | (*data.offset((60 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) as uint32_t)
            << 8 as ::core::ffi::c_int
        | *data.offset((60 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) as uint32_t;
    A = (*ctx).state[0 as ::core::ffi::c_int as usize];
    B = (*ctx).state[1 as ::core::ffi::c_int as usize];
    C = (*ctx).state[2 as ::core::ffi::c_int as usize];
    D = (*ctx).state[3 as ::core::ffi::c_int as usize];
    E = (*ctx).state[4 as ::core::ffi::c_int as usize];
    F = (*ctx).state[5 as ::core::ffi::c_int as usize];
    G = (*ctx).state[6 as ::core::ffi::c_int as usize];
    H = (*ctx).state[7 as ::core::ffi::c_int as usize];
    temp1 = H
        .wrapping_add(
            ((E & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | E << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((E & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | E << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((E & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | E << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(G ^ E & (F ^ G))
        .wrapping_add(0x428a2f98 as uint32_t)
        .wrapping_add(W[0 as ::core::ffi::c_int as usize]);
    temp2 = (((A & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | A << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((A & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | A << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((A & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | A << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(A & B | C & (A | B));
    D = D.wrapping_add(temp1);
    H = temp1.wrapping_add(temp2);
    temp1 = G
        .wrapping_add(
            ((D & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | D << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((D & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | D << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((D & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | D << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(F ^ D & (E ^ F))
        .wrapping_add(0x71374491 as uint32_t)
        .wrapping_add(W[1 as ::core::ffi::c_int as usize]);
    temp2 = (((H & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | H << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((H & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | H << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((H & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | H << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(H & A | B & (H | A));
    C = C.wrapping_add(temp1);
    G = temp1.wrapping_add(temp2);
    temp1 = F
        .wrapping_add(
            ((C & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | C << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((C & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | C << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((C & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | C << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(E ^ C & (D ^ E))
        .wrapping_add(0xb5c0fbcf as uint32_t)
        .wrapping_add(W[2 as ::core::ffi::c_int as usize]);
    temp2 = (((G & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | G << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((G & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | G << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((G & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | G << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(G & H | A & (G | H));
    B = B.wrapping_add(temp1);
    F = temp1.wrapping_add(temp2);
    temp1 = E
        .wrapping_add(
            ((B & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | B << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((B & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | B << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((B & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | B << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(D ^ B & (C ^ D))
        .wrapping_add(0xe9b5dba5 as uint32_t)
        .wrapping_add(W[3 as ::core::ffi::c_int as usize]);
    temp2 = (((F & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | F << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((F & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | F << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((F & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | F << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(F & G | H & (F | G));
    A = A.wrapping_add(temp1);
    E = temp1.wrapping_add(temp2);
    temp1 = D
        .wrapping_add(
            ((A & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | A << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((A & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | A << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((A & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | A << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(C ^ A & (B ^ C))
        .wrapping_add(0x3956c25b as uint32_t)
        .wrapping_add(W[4 as ::core::ffi::c_int as usize]);
    temp2 = (((E & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | E << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((E & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | E << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((E & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | E << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(E & F | G & (E | F));
    H = H.wrapping_add(temp1);
    D = temp1.wrapping_add(temp2);
    temp1 = C
        .wrapping_add(
            ((H & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | H << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((H & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | H << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((H & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | H << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(B ^ H & (A ^ B))
        .wrapping_add(0x59f111f1 as uint32_t)
        .wrapping_add(W[5 as ::core::ffi::c_int as usize]);
    temp2 = (((D & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | D << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((D & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | D << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((D & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | D << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(D & E | F & (D | E));
    G = G.wrapping_add(temp1);
    C = temp1.wrapping_add(temp2);
    temp1 = B
        .wrapping_add(
            ((G & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | G << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((G & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | G << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((G & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | G << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(A ^ G & (H ^ A))
        .wrapping_add(0x923f82a4 as uint32_t)
        .wrapping_add(W[6 as ::core::ffi::c_int as usize]);
    temp2 = (((C & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | C << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((C & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | C << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((C & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | C << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(C & D | E & (C | D));
    F = F.wrapping_add(temp1);
    B = temp1.wrapping_add(temp2);
    temp1 = A
        .wrapping_add(
            ((F & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | F << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((F & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | F << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((F & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | F << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(H ^ F & (G ^ H))
        .wrapping_add(0xab1c5ed5 as uint32_t)
        .wrapping_add(W[7 as ::core::ffi::c_int as usize]);
    temp2 = (((B & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | B << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((B & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | B << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((B & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | B << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(B & C | D & (B | C));
    E = E.wrapping_add(temp1);
    A = temp1.wrapping_add(temp2);
    temp1 = H
        .wrapping_add(
            ((E & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | E << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((E & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | E << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((E & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | E << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(G ^ E & (F ^ G))
        .wrapping_add(0xd807aa98 as uint32_t)
        .wrapping_add(W[8 as ::core::ffi::c_int as usize]);
    temp2 = (((A & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | A << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((A & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | A << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((A & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | A << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(A & B | C & (A | B));
    D = D.wrapping_add(temp1);
    H = temp1.wrapping_add(temp2);
    temp1 = G
        .wrapping_add(
            ((D & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | D << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((D & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | D << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((D & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | D << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(F ^ D & (E ^ F))
        .wrapping_add(0x12835b01 as uint32_t)
        .wrapping_add(W[9 as ::core::ffi::c_int as usize]);
    temp2 = (((H & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | H << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((H & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | H << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((H & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | H << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(H & A | B & (H | A));
    C = C.wrapping_add(temp1);
    G = temp1.wrapping_add(temp2);
    temp1 = F
        .wrapping_add(
            ((C & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | C << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((C & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | C << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((C & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | C << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(E ^ C & (D ^ E))
        .wrapping_add(0x243185be as uint32_t)
        .wrapping_add(W[10 as ::core::ffi::c_int as usize]);
    temp2 = (((G & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | G << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((G & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | G << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((G & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | G << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(G & H | A & (G | H));
    B = B.wrapping_add(temp1);
    F = temp1.wrapping_add(temp2);
    temp1 = E
        .wrapping_add(
            ((B & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | B << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((B & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | B << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((B & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | B << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(D ^ B & (C ^ D))
        .wrapping_add(0x550c7dc3 as uint32_t)
        .wrapping_add(W[11 as ::core::ffi::c_int as usize]);
    temp2 = (((F & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | F << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((F & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | F << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((F & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | F << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(F & G | H & (F | G));
    A = A.wrapping_add(temp1);
    E = temp1.wrapping_add(temp2);
    temp1 = D
        .wrapping_add(
            ((A & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | A << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((A & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | A << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((A & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | A << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(C ^ A & (B ^ C))
        .wrapping_add(0x72be5d74 as uint32_t)
        .wrapping_add(W[12 as ::core::ffi::c_int as usize]);
    temp2 = (((E & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | E << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((E & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | E << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((E & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | E << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(E & F | G & (E | F));
    H = H.wrapping_add(temp1);
    D = temp1.wrapping_add(temp2);
    temp1 = C
        .wrapping_add(
            ((H & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | H << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((H & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | H << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((H & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | H << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(B ^ H & (A ^ B))
        .wrapping_add(0x80deb1fe as uint32_t)
        .wrapping_add(W[13 as ::core::ffi::c_int as usize]);
    temp2 = (((D & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | D << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((D & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | D << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((D & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | D << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(D & E | F & (D | E));
    G = G.wrapping_add(temp1);
    C = temp1.wrapping_add(temp2);
    temp1 = B
        .wrapping_add(
            ((G & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | G << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((G & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | G << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((G & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | G << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(A ^ G & (H ^ A))
        .wrapping_add(0x9bdc06a7 as uint32_t)
        .wrapping_add(W[14 as ::core::ffi::c_int as usize]);
    temp2 = (((C & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | C << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((C & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | C << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((C & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | C << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(C & D | E & (C | D));
    F = F.wrapping_add(temp1);
    B = temp1.wrapping_add(temp2);
    temp1 = A
        .wrapping_add(
            ((F & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | F << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((F & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | F << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((F & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | F << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(H ^ F & (G ^ H))
        .wrapping_add(0xc19bf174 as uint32_t)
        .wrapping_add(W[15 as ::core::ffi::c_int as usize]);
    temp2 = (((B & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | B << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((B & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | B << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((B & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | B << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(B & C | D & (B | C));
    E = E.wrapping_add(temp1);
    A = temp1.wrapping_add(temp2);
    W[16 as ::core::ffi::c_int as usize] = (((W
        [(16 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(16 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(16 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(16 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(16 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(16 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(16 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(16 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(16 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(16 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(16 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(16 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = H
        .wrapping_add(
            ((E & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | E << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((E & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | E << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((E & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | E << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(G ^ E & (F ^ G))
        .wrapping_add(0xe49b69c1 as uint32_t)
        .wrapping_add(W[16 as ::core::ffi::c_int as usize]);
    temp2 = (((A & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | A << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((A & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | A << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((A & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | A << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(A & B | C & (A | B));
    D = D.wrapping_add(temp1);
    H = temp1.wrapping_add(temp2);
    W[17 as ::core::ffi::c_int as usize] = (((W
        [(17 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(17 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(17 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(17 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(17 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(17 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(17 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(17 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(17 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(17 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(17 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(17 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = G
        .wrapping_add(
            ((D & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | D << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((D & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | D << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((D & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | D << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(F ^ D & (E ^ F))
        .wrapping_add(0xefbe4786 as uint32_t)
        .wrapping_add(W[17 as ::core::ffi::c_int as usize]);
    temp2 = (((H & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | H << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((H & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | H << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((H & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | H << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(H & A | B & (H | A));
    C = C.wrapping_add(temp1);
    G = temp1.wrapping_add(temp2);
    W[18 as ::core::ffi::c_int as usize] = (((W
        [(18 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(18 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(18 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(18 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(18 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(18 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(18 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(18 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(18 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(18 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(18 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(18 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = F
        .wrapping_add(
            ((C & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | C << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((C & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | C << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((C & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | C << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(E ^ C & (D ^ E))
        .wrapping_add(0xfc19dc6 as uint32_t)
        .wrapping_add(W[18 as ::core::ffi::c_int as usize]);
    temp2 = (((G & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | G << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((G & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | G << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((G & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | G << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(G & H | A & (G | H));
    B = B.wrapping_add(temp1);
    F = temp1.wrapping_add(temp2);
    W[19 as ::core::ffi::c_int as usize] = (((W
        [(19 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(19 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(19 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(19 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(19 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(19 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(19 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(19 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(19 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(19 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(19 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(19 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = E
        .wrapping_add(
            ((B & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | B << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((B & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | B << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((B & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | B << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(D ^ B & (C ^ D))
        .wrapping_add(0x240ca1cc as uint32_t)
        .wrapping_add(W[19 as ::core::ffi::c_int as usize]);
    temp2 = (((F & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | F << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((F & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | F << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((F & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | F << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(F & G | H & (F | G));
    A = A.wrapping_add(temp1);
    E = temp1.wrapping_add(temp2);
    W[20 as ::core::ffi::c_int as usize] = (((W
        [(20 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(20 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(20 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(20 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(20 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(20 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(20 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(20 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(20 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(20 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(20 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(20 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = D
        .wrapping_add(
            ((A & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | A << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((A & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | A << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((A & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | A << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(C ^ A & (B ^ C))
        .wrapping_add(0x2de92c6f as uint32_t)
        .wrapping_add(W[20 as ::core::ffi::c_int as usize]);
    temp2 = (((E & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | E << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((E & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | E << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((E & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | E << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(E & F | G & (E | F));
    H = H.wrapping_add(temp1);
    D = temp1.wrapping_add(temp2);
    W[21 as ::core::ffi::c_int as usize] = (((W
        [(21 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(21 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(21 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(21 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(21 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(21 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(21 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(21 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(21 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(21 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(21 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(21 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = C
        .wrapping_add(
            ((H & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | H << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((H & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | H << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((H & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | H << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(B ^ H & (A ^ B))
        .wrapping_add(0x4a7484aa as uint32_t)
        .wrapping_add(W[21 as ::core::ffi::c_int as usize]);
    temp2 = (((D & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | D << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((D & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | D << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((D & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | D << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(D & E | F & (D | E));
    G = G.wrapping_add(temp1);
    C = temp1.wrapping_add(temp2);
    W[22 as ::core::ffi::c_int as usize] = (((W
        [(22 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(22 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(22 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(22 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(22 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(22 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(22 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(22 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(22 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(22 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(22 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(22 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = B
        .wrapping_add(
            ((G & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | G << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((G & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | G << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((G & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | G << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(A ^ G & (H ^ A))
        .wrapping_add(0x5cb0a9dc as uint32_t)
        .wrapping_add(W[22 as ::core::ffi::c_int as usize]);
    temp2 = (((C & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | C << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((C & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | C << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((C & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | C << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(C & D | E & (C | D));
    F = F.wrapping_add(temp1);
    B = temp1.wrapping_add(temp2);
    W[23 as ::core::ffi::c_int as usize] = (((W
        [(23 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(23 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(23 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(23 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(23 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(23 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(23 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(23 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(23 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(23 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(23 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(23 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = A
        .wrapping_add(
            ((F & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | F << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((F & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | F << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((F & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | F << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(H ^ F & (G ^ H))
        .wrapping_add(0x76f988da as uint32_t)
        .wrapping_add(W[23 as ::core::ffi::c_int as usize]);
    temp2 = (((B & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | B << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((B & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | B << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((B & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | B << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(B & C | D & (B | C));
    E = E.wrapping_add(temp1);
    A = temp1.wrapping_add(temp2);
    W[24 as ::core::ffi::c_int as usize] = (((W
        [(24 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(24 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(24 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(24 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(24 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(24 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(24 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(24 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(24 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(24 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(24 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(24 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = H
        .wrapping_add(
            ((E & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | E << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((E & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | E << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((E & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | E << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(G ^ E & (F ^ G))
        .wrapping_add(0x983e5152 as uint32_t)
        .wrapping_add(W[24 as ::core::ffi::c_int as usize]);
    temp2 = (((A & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | A << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((A & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | A << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((A & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | A << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(A & B | C & (A | B));
    D = D.wrapping_add(temp1);
    H = temp1.wrapping_add(temp2);
    W[25 as ::core::ffi::c_int as usize] = (((W
        [(25 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(25 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(25 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(25 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(25 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(25 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(25 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(25 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(25 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(25 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(25 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(25 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = G
        .wrapping_add(
            ((D & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | D << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((D & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | D << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((D & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | D << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(F ^ D & (E ^ F))
        .wrapping_add(0xa831c66d as uint32_t)
        .wrapping_add(W[25 as ::core::ffi::c_int as usize]);
    temp2 = (((H & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | H << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((H & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | H << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((H & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | H << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(H & A | B & (H | A));
    C = C.wrapping_add(temp1);
    G = temp1.wrapping_add(temp2);
    W[26 as ::core::ffi::c_int as usize] = (((W
        [(26 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(26 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(26 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(26 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(26 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(26 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(26 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(26 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(26 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(26 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(26 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(26 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = F
        .wrapping_add(
            ((C & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | C << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((C & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | C << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((C & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | C << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(E ^ C & (D ^ E))
        .wrapping_add(0xb00327c8 as uint32_t)
        .wrapping_add(W[26 as ::core::ffi::c_int as usize]);
    temp2 = (((G & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | G << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((G & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | G << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((G & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | G << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(G & H | A & (G | H));
    B = B.wrapping_add(temp1);
    F = temp1.wrapping_add(temp2);
    W[27 as ::core::ffi::c_int as usize] = (((W
        [(27 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(27 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(27 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(27 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(27 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(27 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(27 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(27 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(27 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(27 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(27 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(27 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = E
        .wrapping_add(
            ((B & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | B << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((B & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | B << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((B & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | B << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(D ^ B & (C ^ D))
        .wrapping_add(0xbf597fc7 as uint32_t)
        .wrapping_add(W[27 as ::core::ffi::c_int as usize]);
    temp2 = (((F & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | F << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((F & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | F << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((F & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | F << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(F & G | H & (F | G));
    A = A.wrapping_add(temp1);
    E = temp1.wrapping_add(temp2);
    W[28 as ::core::ffi::c_int as usize] = (((W
        [(28 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(28 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(28 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(28 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(28 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(28 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(28 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(28 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(28 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(28 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(28 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(28 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = D
        .wrapping_add(
            ((A & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | A << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((A & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | A << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((A & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | A << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(C ^ A & (B ^ C))
        .wrapping_add(0xc6e00bf3 as uint32_t)
        .wrapping_add(W[28 as ::core::ffi::c_int as usize]);
    temp2 = (((E & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | E << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((E & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | E << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((E & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | E << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(E & F | G & (E | F));
    H = H.wrapping_add(temp1);
    D = temp1.wrapping_add(temp2);
    W[29 as ::core::ffi::c_int as usize] = (((W
        [(29 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(29 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(29 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(29 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(29 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(29 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(29 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(29 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(29 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(29 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(29 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(29 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = C
        .wrapping_add(
            ((H & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | H << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((H & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | H << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((H & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | H << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(B ^ H & (A ^ B))
        .wrapping_add(0xd5a79147 as uint32_t)
        .wrapping_add(W[29 as ::core::ffi::c_int as usize]);
    temp2 = (((D & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | D << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((D & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | D << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((D & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | D << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(D & E | F & (D | E));
    G = G.wrapping_add(temp1);
    C = temp1.wrapping_add(temp2);
    W[30 as ::core::ffi::c_int as usize] = (((W
        [(30 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(30 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(30 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(30 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(30 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(30 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(30 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(30 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(30 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(30 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(30 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(30 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = B
        .wrapping_add(
            ((G & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | G << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((G & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | G << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((G & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | G << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(A ^ G & (H ^ A))
        .wrapping_add(0x6ca6351 as uint32_t)
        .wrapping_add(W[30 as ::core::ffi::c_int as usize]);
    temp2 = (((C & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | C << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((C & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | C << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((C & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | C << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(C & D | E & (C | D));
    F = F.wrapping_add(temp1);
    B = temp1.wrapping_add(temp2);
    W[31 as ::core::ffi::c_int as usize] = (((W
        [(31 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(31 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(31 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(31 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(31 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(31 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(31 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(31 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(31 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(31 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(31 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(31 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = A
        .wrapping_add(
            ((F & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | F << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((F & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | F << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((F & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | F << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(H ^ F & (G ^ H))
        .wrapping_add(0x14292967 as uint32_t)
        .wrapping_add(W[31 as ::core::ffi::c_int as usize]);
    temp2 = (((B & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | B << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((B & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | B << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((B & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | B << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(B & C | D & (B | C));
    E = E.wrapping_add(temp1);
    A = temp1.wrapping_add(temp2);
    W[32 as ::core::ffi::c_int as usize] = (((W
        [(32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(32 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(32 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(32 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(32 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(32 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(32 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = H
        .wrapping_add(
            ((E & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | E << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((E & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | E << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((E & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | E << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(G ^ E & (F ^ G))
        .wrapping_add(0x27b70a85 as uint32_t)
        .wrapping_add(W[32 as ::core::ffi::c_int as usize]);
    temp2 = (((A & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | A << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((A & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | A << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((A & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | A << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(A & B | C & (A | B));
    D = D.wrapping_add(temp1);
    H = temp1.wrapping_add(temp2);
    W[33 as ::core::ffi::c_int as usize] = (((W
        [(33 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(33 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(33 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(33 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(33 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(33 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(33 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(33 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(33 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(33 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(33 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(33 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = G
        .wrapping_add(
            ((D & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | D << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((D & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | D << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((D & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | D << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(F ^ D & (E ^ F))
        .wrapping_add(0x2e1b2138 as uint32_t)
        .wrapping_add(W[33 as ::core::ffi::c_int as usize]);
    temp2 = (((H & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | H << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((H & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | H << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((H & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | H << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(H & A | B & (H | A));
    C = C.wrapping_add(temp1);
    G = temp1.wrapping_add(temp2);
    W[34 as ::core::ffi::c_int as usize] = (((W
        [(34 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(34 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(34 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(34 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(34 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(34 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(34 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(34 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(34 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(34 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(34 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(34 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = F
        .wrapping_add(
            ((C & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | C << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((C & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | C << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((C & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | C << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(E ^ C & (D ^ E))
        .wrapping_add(0x4d2c6dfc as uint32_t)
        .wrapping_add(W[34 as ::core::ffi::c_int as usize]);
    temp2 = (((G & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | G << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((G & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | G << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((G & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | G << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(G & H | A & (G | H));
    B = B.wrapping_add(temp1);
    F = temp1.wrapping_add(temp2);
    W[35 as ::core::ffi::c_int as usize] = (((W
        [(35 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(35 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(35 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(35 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(35 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(35 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(35 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(35 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(35 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(35 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(35 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(35 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = E
        .wrapping_add(
            ((B & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | B << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((B & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | B << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((B & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | B << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(D ^ B & (C ^ D))
        .wrapping_add(0x53380d13 as uint32_t)
        .wrapping_add(W[35 as ::core::ffi::c_int as usize]);
    temp2 = (((F & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | F << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((F & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | F << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((F & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | F << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(F & G | H & (F | G));
    A = A.wrapping_add(temp1);
    E = temp1.wrapping_add(temp2);
    W[36 as ::core::ffi::c_int as usize] = (((W
        [(36 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(36 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(36 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(36 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(36 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(36 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(36 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(36 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(36 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(36 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(36 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(36 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = D
        .wrapping_add(
            ((A & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | A << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((A & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | A << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((A & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | A << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(C ^ A & (B ^ C))
        .wrapping_add(0x650a7354 as uint32_t)
        .wrapping_add(W[36 as ::core::ffi::c_int as usize]);
    temp2 = (((E & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | E << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((E & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | E << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((E & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | E << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(E & F | G & (E | F));
    H = H.wrapping_add(temp1);
    D = temp1.wrapping_add(temp2);
    W[37 as ::core::ffi::c_int as usize] = (((W
        [(37 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(37 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(37 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(37 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(37 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(37 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(37 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(37 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(37 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(37 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(37 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(37 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = C
        .wrapping_add(
            ((H & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | H << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((H & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | H << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((H & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | H << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(B ^ H & (A ^ B))
        .wrapping_add(0x766a0abb as uint32_t)
        .wrapping_add(W[37 as ::core::ffi::c_int as usize]);
    temp2 = (((D & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | D << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((D & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | D << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((D & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | D << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(D & E | F & (D | E));
    G = G.wrapping_add(temp1);
    C = temp1.wrapping_add(temp2);
    W[38 as ::core::ffi::c_int as usize] = (((W
        [(38 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(38 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(38 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(38 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(38 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(38 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(38 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(38 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(38 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(38 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(38 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(38 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = B
        .wrapping_add(
            ((G & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | G << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((G & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | G << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((G & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | G << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(A ^ G & (H ^ A))
        .wrapping_add(0x81c2c92e as uint32_t)
        .wrapping_add(W[38 as ::core::ffi::c_int as usize]);
    temp2 = (((C & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | C << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((C & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | C << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((C & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | C << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(C & D | E & (C | D));
    F = F.wrapping_add(temp1);
    B = temp1.wrapping_add(temp2);
    W[39 as ::core::ffi::c_int as usize] = (((W
        [(39 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(39 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(39 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(39 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(39 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(39 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(39 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(39 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(39 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(39 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(39 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(39 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = A
        .wrapping_add(
            ((F & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | F << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((F & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | F << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((F & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | F << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(H ^ F & (G ^ H))
        .wrapping_add(0x92722c85 as uint32_t)
        .wrapping_add(W[39 as ::core::ffi::c_int as usize]);
    temp2 = (((B & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | B << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((B & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | B << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((B & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | B << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(B & C | D & (B | C));
    E = E.wrapping_add(temp1);
    A = temp1.wrapping_add(temp2);
    W[40 as ::core::ffi::c_int as usize] = (((W
        [(40 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(40 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(40 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(40 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(40 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(40 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(40 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(40 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(40 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(40 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(40 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(40 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = H
        .wrapping_add(
            ((E & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | E << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((E & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | E << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((E & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | E << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(G ^ E & (F ^ G))
        .wrapping_add(0xa2bfe8a1 as uint32_t)
        .wrapping_add(W[40 as ::core::ffi::c_int as usize]);
    temp2 = (((A & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | A << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((A & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | A << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((A & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | A << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(A & B | C & (A | B));
    D = D.wrapping_add(temp1);
    H = temp1.wrapping_add(temp2);
    W[41 as ::core::ffi::c_int as usize] = (((W
        [(41 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(41 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(41 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(41 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(41 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(41 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(41 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(41 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(41 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(41 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(41 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(41 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = G
        .wrapping_add(
            ((D & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | D << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((D & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | D << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((D & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | D << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(F ^ D & (E ^ F))
        .wrapping_add(0xa81a664b as uint32_t)
        .wrapping_add(W[41 as ::core::ffi::c_int as usize]);
    temp2 = (((H & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | H << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((H & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | H << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((H & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | H << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(H & A | B & (H | A));
    C = C.wrapping_add(temp1);
    G = temp1.wrapping_add(temp2);
    W[42 as ::core::ffi::c_int as usize] = (((W
        [(42 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(42 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(42 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(42 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(42 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(42 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(42 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(42 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(42 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(42 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(42 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(42 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = F
        .wrapping_add(
            ((C & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | C << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((C & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | C << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((C & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | C << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(E ^ C & (D ^ E))
        .wrapping_add(0xc24b8b70 as uint32_t)
        .wrapping_add(W[42 as ::core::ffi::c_int as usize]);
    temp2 = (((G & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | G << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((G & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | G << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((G & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | G << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(G & H | A & (G | H));
    B = B.wrapping_add(temp1);
    F = temp1.wrapping_add(temp2);
    W[43 as ::core::ffi::c_int as usize] = (((W
        [(43 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(43 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(43 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(43 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(43 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(43 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(43 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(43 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(43 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(43 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(43 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(43 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = E
        .wrapping_add(
            ((B & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | B << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((B & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | B << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((B & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | B << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(D ^ B & (C ^ D))
        .wrapping_add(0xc76c51a3 as uint32_t)
        .wrapping_add(W[43 as ::core::ffi::c_int as usize]);
    temp2 = (((F & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | F << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((F & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | F << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((F & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | F << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(F & G | H & (F | G));
    A = A.wrapping_add(temp1);
    E = temp1.wrapping_add(temp2);
    W[44 as ::core::ffi::c_int as usize] = (((W
        [(44 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(44 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(44 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(44 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(44 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(44 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(44 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(44 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(44 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(44 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(44 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(44 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = D
        .wrapping_add(
            ((A & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | A << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((A & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | A << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((A & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | A << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(C ^ A & (B ^ C))
        .wrapping_add(0xd192e819 as uint32_t)
        .wrapping_add(W[44 as ::core::ffi::c_int as usize]);
    temp2 = (((E & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | E << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((E & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | E << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((E & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | E << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(E & F | G & (E | F));
    H = H.wrapping_add(temp1);
    D = temp1.wrapping_add(temp2);
    W[45 as ::core::ffi::c_int as usize] = (((W
        [(45 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(45 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(45 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(45 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(45 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(45 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(45 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(45 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(45 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(45 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(45 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(45 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = C
        .wrapping_add(
            ((H & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | H << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((H & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | H << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((H & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | H << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(B ^ H & (A ^ B))
        .wrapping_add(0xd6990624 as uint32_t)
        .wrapping_add(W[45 as ::core::ffi::c_int as usize]);
    temp2 = (((D & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | D << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((D & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | D << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((D & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | D << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(D & E | F & (D | E));
    G = G.wrapping_add(temp1);
    C = temp1.wrapping_add(temp2);
    W[46 as ::core::ffi::c_int as usize] = (((W
        [(46 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(46 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(46 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(46 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(46 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(46 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(46 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(46 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(46 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(46 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(46 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(46 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = B
        .wrapping_add(
            ((G & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | G << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((G & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | G << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((G & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | G << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(A ^ G & (H ^ A))
        .wrapping_add(0xf40e3585 as uint32_t)
        .wrapping_add(W[46 as ::core::ffi::c_int as usize]);
    temp2 = (((C & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | C << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((C & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | C << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((C & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | C << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(C & D | E & (C | D));
    F = F.wrapping_add(temp1);
    B = temp1.wrapping_add(temp2);
    W[47 as ::core::ffi::c_int as usize] = (((W
        [(47 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(47 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(47 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(47 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(47 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(47 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(47 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(47 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(47 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(47 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(47 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(47 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = A
        .wrapping_add(
            ((F & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | F << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((F & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | F << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((F & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | F << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(H ^ F & (G ^ H))
        .wrapping_add(0x106aa070 as uint32_t)
        .wrapping_add(W[47 as ::core::ffi::c_int as usize]);
    temp2 = (((B & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | B << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((B & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | B << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((B & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | B << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(B & C | D & (B | C));
    E = E.wrapping_add(temp1);
    A = temp1.wrapping_add(temp2);
    W[48 as ::core::ffi::c_int as usize] = (((W
        [(48 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(48 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(48 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(48 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(48 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(48 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(48 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(48 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(48 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(48 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(48 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(48 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = H
        .wrapping_add(
            ((E & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | E << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((E & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | E << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((E & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | E << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(G ^ E & (F ^ G))
        .wrapping_add(0x19a4c116 as uint32_t)
        .wrapping_add(W[48 as ::core::ffi::c_int as usize]);
    temp2 = (((A & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | A << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((A & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | A << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((A & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | A << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(A & B | C & (A | B));
    D = D.wrapping_add(temp1);
    H = temp1.wrapping_add(temp2);
    W[49 as ::core::ffi::c_int as usize] = (((W
        [(49 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(49 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(49 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(49 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(49 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(49 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(49 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(49 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(49 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(49 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(49 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(49 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = G
        .wrapping_add(
            ((D & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | D << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((D & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | D << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((D & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | D << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(F ^ D & (E ^ F))
        .wrapping_add(0x1e376c08 as uint32_t)
        .wrapping_add(W[49 as ::core::ffi::c_int as usize]);
    temp2 = (((H & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | H << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((H & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | H << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((H & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | H << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(H & A | B & (H | A));
    C = C.wrapping_add(temp1);
    G = temp1.wrapping_add(temp2);
    W[50 as ::core::ffi::c_int as usize] = (((W
        [(50 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(50 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(50 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(50 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(50 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(50 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(50 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(50 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(50 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(50 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(50 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(50 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = F
        .wrapping_add(
            ((C & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | C << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((C & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | C << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((C & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | C << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(E ^ C & (D ^ E))
        .wrapping_add(0x2748774c as uint32_t)
        .wrapping_add(W[50 as ::core::ffi::c_int as usize]);
    temp2 = (((G & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | G << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((G & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | G << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((G & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | G << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(G & H | A & (G | H));
    B = B.wrapping_add(temp1);
    F = temp1.wrapping_add(temp2);
    W[51 as ::core::ffi::c_int as usize] = (((W
        [(51 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(51 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(51 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(51 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(51 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(51 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(51 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(51 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(51 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(51 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(51 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(51 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = E
        .wrapping_add(
            ((B & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | B << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((B & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | B << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((B & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | B << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(D ^ B & (C ^ D))
        .wrapping_add(0x34b0bcb5 as uint32_t)
        .wrapping_add(W[51 as ::core::ffi::c_int as usize]);
    temp2 = (((F & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | F << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((F & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | F << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((F & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | F << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(F & G | H & (F | G));
    A = A.wrapping_add(temp1);
    E = temp1.wrapping_add(temp2);
    W[52 as ::core::ffi::c_int as usize] = (((W
        [(52 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(52 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(52 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(52 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(52 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(52 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(52 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(52 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(52 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(52 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(52 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(52 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = D
        .wrapping_add(
            ((A & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | A << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((A & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | A << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((A & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | A << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(C ^ A & (B ^ C))
        .wrapping_add(0x391c0cb3 as uint32_t)
        .wrapping_add(W[52 as ::core::ffi::c_int as usize]);
    temp2 = (((E & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | E << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((E & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | E << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((E & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | E << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(E & F | G & (E | F));
    H = H.wrapping_add(temp1);
    D = temp1.wrapping_add(temp2);
    W[53 as ::core::ffi::c_int as usize] = (((W
        [(53 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(53 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(53 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(53 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(53 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(53 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(53 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(53 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(53 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(53 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(53 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(53 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = C
        .wrapping_add(
            ((H & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | H << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((H & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | H << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((H & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | H << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(B ^ H & (A ^ B))
        .wrapping_add(0x4ed8aa4a as uint32_t)
        .wrapping_add(W[53 as ::core::ffi::c_int as usize]);
    temp2 = (((D & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | D << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((D & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | D << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((D & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | D << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(D & E | F & (D | E));
    G = G.wrapping_add(temp1);
    C = temp1.wrapping_add(temp2);
    W[54 as ::core::ffi::c_int as usize] = (((W
        [(54 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(54 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(54 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(54 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(54 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(54 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(54 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(54 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(54 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(54 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(54 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(54 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = B
        .wrapping_add(
            ((G & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | G << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((G & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | G << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((G & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | G << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(A ^ G & (H ^ A))
        .wrapping_add(0x5b9cca4f as uint32_t)
        .wrapping_add(W[54 as ::core::ffi::c_int as usize]);
    temp2 = (((C & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | C << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((C & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | C << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((C & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | C << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(C & D | E & (C | D));
    F = F.wrapping_add(temp1);
    B = temp1.wrapping_add(temp2);
    W[55 as ::core::ffi::c_int as usize] = (((W
        [(55 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(55 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(55 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(55 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(55 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(55 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(55 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(55 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(55 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(55 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(55 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(55 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = A
        .wrapping_add(
            ((F & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | F << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((F & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | F << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((F & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | F << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(H ^ F & (G ^ H))
        .wrapping_add(0x682e6ff3 as uint32_t)
        .wrapping_add(W[55 as ::core::ffi::c_int as usize]);
    temp2 = (((B & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | B << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((B & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | B << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((B & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | B << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(B & C | D & (B | C));
    E = E.wrapping_add(temp1);
    A = temp1.wrapping_add(temp2);
    W[56 as ::core::ffi::c_int as usize] = (((W
        [(56 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(56 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(56 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(56 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(56 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(56 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(56 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(56 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(56 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(56 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(56 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(56 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = H
        .wrapping_add(
            ((E & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | E << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((E & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | E << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((E & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | E << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(G ^ E & (F ^ G))
        .wrapping_add(0x748f82ee as uint32_t)
        .wrapping_add(W[56 as ::core::ffi::c_int as usize]);
    temp2 = (((A & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | A << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((A & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | A << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((A & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | A << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(A & B | C & (A | B));
    D = D.wrapping_add(temp1);
    H = temp1.wrapping_add(temp2);
    W[57 as ::core::ffi::c_int as usize] = (((W
        [(57 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(57 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(57 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(57 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(57 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(57 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(57 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(57 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(57 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(57 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(57 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(57 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = G
        .wrapping_add(
            ((D & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | D << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((D & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | D << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((D & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | D << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(F ^ D & (E ^ F))
        .wrapping_add(0x78a5636f as uint32_t)
        .wrapping_add(W[57 as ::core::ffi::c_int as usize]);
    temp2 = (((H & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | H << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((H & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | H << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((H & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | H << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(H & A | B & (H | A));
    C = C.wrapping_add(temp1);
    G = temp1.wrapping_add(temp2);
    W[58 as ::core::ffi::c_int as usize] = (((W
        [(58 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(58 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(58 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(58 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(58 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(58 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(58 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(58 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(58 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(58 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(58 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(58 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = F
        .wrapping_add(
            ((C & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | C << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((C & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | C << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((C & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | C << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(E ^ C & (D ^ E))
        .wrapping_add(0x84c87814 as uint32_t)
        .wrapping_add(W[58 as ::core::ffi::c_int as usize]);
    temp2 = (((G & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | G << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((G & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | G << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((G & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | G << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(G & H | A & (G | H));
    B = B.wrapping_add(temp1);
    F = temp1.wrapping_add(temp2);
    W[59 as ::core::ffi::c_int as usize] = (((W
        [(59 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(59 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(59 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(59 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(59 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(59 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(59 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(59 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(59 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(59 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(59 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(59 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = E
        .wrapping_add(
            ((B & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | B << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((B & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | B << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((B & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | B << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(D ^ B & (C ^ D))
        .wrapping_add(0x8cc70208 as uint32_t)
        .wrapping_add(W[59 as ::core::ffi::c_int as usize]);
    temp2 = (((F & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | F << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((F & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | F << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((F & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | F << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(F & G | H & (F | G));
    A = A.wrapping_add(temp1);
    E = temp1.wrapping_add(temp2);
    W[60 as ::core::ffi::c_int as usize] = (((W
        [(60 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(60 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(60 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(60 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(60 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(60 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(60 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(60 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(60 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(60 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(60 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(60 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = D
        .wrapping_add(
            ((A & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | A << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((A & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | A << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((A & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | A << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(C ^ A & (B ^ C))
        .wrapping_add(0x90befffa as uint32_t)
        .wrapping_add(W[60 as ::core::ffi::c_int as usize]);
    temp2 = (((E & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | E << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((E & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | E << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((E & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | E << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(E & F | G & (E | F));
    H = H.wrapping_add(temp1);
    D = temp1.wrapping_add(temp2);
    W[61 as ::core::ffi::c_int as usize] = (((W
        [(61 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(61 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(61 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(61 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(61 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(61 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(61 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(61 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(61 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(61 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(61 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(61 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = C
        .wrapping_add(
            ((H & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | H << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((H & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | H << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((H & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | H << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(B ^ H & (A ^ B))
        .wrapping_add(0xa4506ceb as uint32_t)
        .wrapping_add(W[61 as ::core::ffi::c_int as usize]);
    temp2 = (((D & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | D << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((D & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | D << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((D & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | D << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(D & E | F & (D | E));
    G = G.wrapping_add(temp1);
    C = temp1.wrapping_add(temp2);
    W[62 as ::core::ffi::c_int as usize] = (((W
        [(62 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(62 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(62 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(62 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(62 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(62 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(62 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(62 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(62 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(62 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(62 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(62 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = B
        .wrapping_add(
            ((G & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | G << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((G & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | G << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((G & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | G << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(A ^ G & (H ^ A))
        .wrapping_add(0xbef9a3f7 as uint32_t)
        .wrapping_add(W[62 as ::core::ffi::c_int as usize]);
    temp2 = (((C & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | C << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((C & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | C << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((C & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | C << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(C & D | E & (C | D));
    F = F.wrapping_add(temp1);
    B = temp1.wrapping_add(temp2);
    W[63 as ::core::ffi::c_int as usize] = (((W
        [(63 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
        & 0xffffffff as uint32_t)
        >> 17 as ::core::ffi::c_int
        | W[(63 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            << 32 as ::core::ffi::c_int - 17 as ::core::ffi::c_int)
        ^ ((W[(63 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 19 as ::core::ffi::c_int
            | W[(63 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                << 32 as ::core::ffi::c_int - 19 as ::core::ffi::c_int)
        ^ (W[(63 as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
            & 0xffffffff as uint32_t)
            >> 10 as ::core::ffi::c_int)
        .wrapping_add(W[(63 as ::core::ffi::c_int - 7 as ::core::ffi::c_int) as usize])
        .wrapping_add(
            ((W[(63 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                & 0xffffffff as uint32_t)
                >> 7 as ::core::ffi::c_int
                | W[(63 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    << 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
                ^ ((W[(63 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 18 as ::core::ffi::c_int
                    | W[(63 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                        << 32 as ::core::ffi::c_int - 18 as ::core::ffi::c_int)
                ^ (W[(63 as ::core::ffi::c_int - 15 as ::core::ffi::c_int) as usize]
                    & 0xffffffff as uint32_t)
                    >> 3 as ::core::ffi::c_int,
        )
        .wrapping_add(W[(63 as ::core::ffi::c_int - 16 as ::core::ffi::c_int) as usize]);
    temp1 = A
        .wrapping_add(
            ((F & 0xffffffff as uint32_t) >> 6 as ::core::ffi::c_int
                | F << 32 as ::core::ffi::c_int - 6 as ::core::ffi::c_int)
                ^ ((F & 0xffffffff as uint32_t) >> 11 as ::core::ffi::c_int
                    | F << 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int)
                ^ ((F & 0xffffffff as uint32_t) >> 25 as ::core::ffi::c_int
                    | F << 32 as ::core::ffi::c_int - 25 as ::core::ffi::c_int),
        )
        .wrapping_add(H ^ F & (G ^ H))
        .wrapping_add(0xc67178f2 as uint32_t)
        .wrapping_add(W[63 as ::core::ffi::c_int as usize]);
    temp2 = (((B & 0xffffffff as uint32_t) >> 2 as ::core::ffi::c_int
        | B << 32 as ::core::ffi::c_int - 2 as ::core::ffi::c_int)
        ^ ((B & 0xffffffff as uint32_t) >> 13 as ::core::ffi::c_int
            | B << 32 as ::core::ffi::c_int - 13 as ::core::ffi::c_int)
        ^ ((B & 0xffffffff as uint32_t) >> 22 as ::core::ffi::c_int
            | B << 32 as ::core::ffi::c_int - 22 as ::core::ffi::c_int))
        .wrapping_add(B & C | D & (B | C));
    E = E.wrapping_add(temp1);
    A = temp1.wrapping_add(temp2);
    (*ctx).state[0 as ::core::ffi::c_int as usize] =
        (*ctx).state[0 as ::core::ffi::c_int as usize].wrapping_add(A);
    (*ctx).state[1 as ::core::ffi::c_int as usize] =
        (*ctx).state[1 as ::core::ffi::c_int as usize].wrapping_add(B);
    (*ctx).state[2 as ::core::ffi::c_int as usize] =
        (*ctx).state[2 as ::core::ffi::c_int as usize].wrapping_add(C);
    (*ctx).state[3 as ::core::ffi::c_int as usize] =
        (*ctx).state[3 as ::core::ffi::c_int as usize].wrapping_add(D);
    (*ctx).state[4 as ::core::ffi::c_int as usize] =
        (*ctx).state[4 as ::core::ffi::c_int as usize].wrapping_add(E);
    (*ctx).state[5 as ::core::ffi::c_int as usize] =
        (*ctx).state[5 as ::core::ffi::c_int as usize].wrapping_add(F);
    (*ctx).state[6 as ::core::ffi::c_int as usize] =
        (*ctx).state[6 as ::core::ffi::c_int as usize].wrapping_add(G);
    (*ctx).state[7 as ::core::ffi::c_int as usize] =
        (*ctx).state[7 as ::core::ffi::c_int as usize].wrapping_add(H);
}
#[no_mangle]
pub unsafe extern "C" fn sha256_update(
    mut ctx: *mut context_sha256_T,
    mut input: *const uint8_t,
    mut length: size_t,
) {
    if length == 0 as size_t {
        return;
    }
    let mut left: uint32_t = (*ctx).total[0 as ::core::ffi::c_int as usize]
        & (SHA256_BUFFER_SIZE - 1 as ::core::ffi::c_int) as uint32_t;
    (*ctx).total[0 as ::core::ffi::c_int as usize] =
        (*ctx).total[0 as ::core::ffi::c_int as usize].wrapping_add(length as uint32_t);
    (*ctx).total[0 as ::core::ffi::c_int as usize] =
        ((*ctx).total[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_uint
            & 0xffffffff as ::core::ffi::c_uint) as uint32_t;
    if ((*ctx).total[0 as ::core::ffi::c_int as usize] as size_t) < length {
        (*ctx).total[1 as ::core::ffi::c_int as usize] =
            (*ctx).total[1 as ::core::ffi::c_int as usize].wrapping_add(1);
    }
    let mut fill: size_t = (SHA256_BUFFER_SIZE as uint32_t).wrapping_sub(left) as size_t;
    if left != 0 && length >= fill {
        memcpy(
            (&raw mut (*ctx).buffer as *mut uint8_t).offset(left as isize)
                as *mut ::core::ffi::c_void,
            input as *const ::core::ffi::c_void,
            fill,
        );
        sha256_process(
            ctx,
            &raw mut (*ctx).buffer as *mut uint8_t as *const uint8_t,
        );
        length = length.wrapping_sub(fill);
        input = input.offset(fill as isize);
        left = 0 as uint32_t;
    }
    while length >= SHA256_BUFFER_SIZE as size_t {
        sha256_process(ctx, input as *const uint8_t);
        length = length.wrapping_sub(SHA256_BUFFER_SIZE as size_t);
        input = input.offset(SHA256_BUFFER_SIZE as isize);
    }
    if length != 0 {
        memcpy(
            (&raw mut (*ctx).buffer as *mut uint8_t).offset(left as isize)
                as *mut ::core::ffi::c_void,
            input as *const ::core::ffi::c_void,
            length,
        );
    }
}
static sha256_padding: GlobalCell<[uint8_t; 64]> = GlobalCell::new([
    0x80 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
    0 as uint8_t,
]);
#[no_mangle]
pub unsafe extern "C" fn sha256_finish(mut ctx: *mut context_sha256_T, mut digest: *mut uint8_t) {
    let mut high: uint32_t = (*ctx).total[0 as ::core::ffi::c_int as usize]
        >> 29 as ::core::ffi::c_int
        | (*ctx).total[1 as ::core::ffi::c_int as usize] << 3 as ::core::ffi::c_int;
    let mut low: uint32_t =
        (*ctx).total[0 as ::core::ffi::c_int as usize] << 3 as ::core::ffi::c_int;
    let mut msglen: [uint8_t; 8] = [0; 8];
    msglen[0 as ::core::ffi::c_int as usize] = (high >> 24 as ::core::ffi::c_int) as uint8_t;
    msglen[(0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as usize] =
        (high >> 16 as ::core::ffi::c_int) as uint8_t;
    msglen[(0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as usize] =
        (high >> 8 as ::core::ffi::c_int) as uint8_t;
    msglen[(0 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as usize] = high as uint8_t;
    msglen[4 as ::core::ffi::c_int as usize] = (low >> 24 as ::core::ffi::c_int) as uint8_t;
    msglen[(4 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as usize] =
        (low >> 16 as ::core::ffi::c_int) as uint8_t;
    msglen[(4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as usize] =
        (low >> 8 as ::core::ffi::c_int) as uint8_t;
    msglen[(4 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as usize] = low as uint8_t;
    let mut last: uint32_t = (*ctx).total[0 as ::core::ffi::c_int as usize] & 0x3f as uint32_t;
    let mut padn: uint32_t = if last < 56 as uint32_t {
        (56 as uint32_t).wrapping_sub(last)
    } else {
        (120 as uint32_t).wrapping_sub(last)
    };
    sha256_update(ctx, sha256_padding.ptr() as *mut uint8_t, padn as size_t);
    sha256_update(ctx, &raw mut msglen as *mut uint8_t, 8 as size_t);
    *digest.offset(0 as ::core::ffi::c_int as isize) =
        ((*ctx).state[0 as ::core::ffi::c_int as usize] >> 24 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
        ((*ctx).state[0 as ::core::ffi::c_int as usize] >> 16 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((0 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) =
        ((*ctx).state[0 as ::core::ffi::c_int as usize] >> 8 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((0 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) =
        (*ctx).state[0 as ::core::ffi::c_int as usize] as uint8_t;
    *digest.offset(4 as ::core::ffi::c_int as isize) =
        ((*ctx).state[1 as ::core::ffi::c_int as usize] >> 24 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((4 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
        ((*ctx).state[1 as ::core::ffi::c_int as usize] >> 16 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((4 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) =
        ((*ctx).state[1 as ::core::ffi::c_int as usize] >> 8 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((4 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) =
        (*ctx).state[1 as ::core::ffi::c_int as usize] as uint8_t;
    *digest.offset(8 as ::core::ffi::c_int as isize) =
        ((*ctx).state[2 as ::core::ffi::c_int as usize] >> 24 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((8 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
        ((*ctx).state[2 as ::core::ffi::c_int as usize] >> 16 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((8 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) =
        ((*ctx).state[2 as ::core::ffi::c_int as usize] >> 8 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((8 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) =
        (*ctx).state[2 as ::core::ffi::c_int as usize] as uint8_t;
    *digest.offset(12 as ::core::ffi::c_int as isize) =
        ((*ctx).state[3 as ::core::ffi::c_int as usize] >> 24 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((12 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
        ((*ctx).state[3 as ::core::ffi::c_int as usize] >> 16 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((12 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) =
        ((*ctx).state[3 as ::core::ffi::c_int as usize] >> 8 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((12 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) =
        (*ctx).state[3 as ::core::ffi::c_int as usize] as uint8_t;
    *digest.offset(16 as ::core::ffi::c_int as isize) =
        ((*ctx).state[4 as ::core::ffi::c_int as usize] >> 24 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((16 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
        ((*ctx).state[4 as ::core::ffi::c_int as usize] >> 16 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((16 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) =
        ((*ctx).state[4 as ::core::ffi::c_int as usize] >> 8 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((16 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) =
        (*ctx).state[4 as ::core::ffi::c_int as usize] as uint8_t;
    *digest.offset(20 as ::core::ffi::c_int as isize) =
        ((*ctx).state[5 as ::core::ffi::c_int as usize] >> 24 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((20 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
        ((*ctx).state[5 as ::core::ffi::c_int as usize] >> 16 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((20 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) =
        ((*ctx).state[5 as ::core::ffi::c_int as usize] >> 8 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((20 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) =
        (*ctx).state[5 as ::core::ffi::c_int as usize] as uint8_t;
    *digest.offset(24 as ::core::ffi::c_int as isize) =
        ((*ctx).state[6 as ::core::ffi::c_int as usize] >> 24 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((24 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
        ((*ctx).state[6 as ::core::ffi::c_int as usize] >> 16 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((24 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) =
        ((*ctx).state[6 as ::core::ffi::c_int as usize] >> 8 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((24 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) =
        (*ctx).state[6 as ::core::ffi::c_int as usize] as uint8_t;
    *digest.offset(28 as ::core::ffi::c_int as isize) =
        ((*ctx).state[7 as ::core::ffi::c_int as usize] >> 24 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((28 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize) =
        ((*ctx).state[7 as ::core::ffi::c_int as usize] >> 16 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((28 as ::core::ffi::c_int + 2 as ::core::ffi::c_int) as isize) =
        ((*ctx).state[7 as ::core::ffi::c_int as usize] >> 8 as ::core::ffi::c_int) as uint8_t;
    *digest.offset((28 as ::core::ffi::c_int + 3 as ::core::ffi::c_int) as isize) =
        (*ctx).state[7 as ::core::ffi::c_int as usize] as uint8_t;
}
pub const SHA_STEP: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn sha256_bytes(
    mut buf: *const uint8_t,
    mut buf_len: size_t,
    mut salt: *const uint8_t,
    mut salt_len: size_t,
) -> *const ::core::ffi::c_char {
    static hexit: GlobalCell<[::core::ffi::c_char; 65]> = GlobalCell::new([0; 65]);
    sha256_self_test();
    let mut ctx: context_sha256_T = context_sha256_T {
        total: [0; 2],
        state: [0; 8],
        buffer: [0; 64],
    };
    sha256_start(&raw mut ctx);
    sha256_update(&raw mut ctx, buf, buf_len);
    if !salt.is_null() {
        sha256_update(&raw mut ctx, salt, salt_len);
    }
    let mut sha256sum: [uint8_t; 32] = [0; 32];
    sha256_finish(&raw mut ctx, &raw mut sha256sum as *mut uint8_t);
    let mut j: size_t = 0 as size_t;
    while j < SHA256_SUM_SIZE as size_t {
        snprintf(
            (hexit.ptr() as *mut ::core::ffi::c_char)
                .offset(j.wrapping_mul(SHA_STEP as size_t) as isize),
            (SHA_STEP + 1 as ::core::ffi::c_int) as size_t,
            b"%02x\0".as_ptr() as *const ::core::ffi::c_char,
            sha256sum[j as usize] as ::core::ffi::c_int,
        );
        j = j.wrapping_add(1);
    }
    (*hexit.ptr())
        [::core::mem::size_of::<[::core::ffi::c_char; 65]>().wrapping_sub(1 as usize) as usize] =
        NUL as ::core::ffi::c_char;
    return hexit.ptr() as *mut ::core::ffi::c_char;
}
static sha_self_test_msg: GlobalCell<[*mut ::core::ffi::c_char; 3]> = GlobalCell::new([
    b"abc\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq\0".as_ptr()
        as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ::core::ptr::null_mut::<::core::ffi::c_char>(),
]);
static sha_self_test_vector: GlobalCell<[*mut ::core::ffi::c_char; 3]> = GlobalCell::new([
    b"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\0".as_ptr()
        as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1\0".as_ptr()
        as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0\0".as_ptr()
        as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
]);
#[no_mangle]
pub unsafe extern "C" fn sha256_self_test() -> bool {
    let mut output: [::core::ffi::c_char; 65] = [0; 65];
    let mut ctx: context_sha256_T = context_sha256_T {
        total: [0; 2],
        state: [0; 8],
        buffer: [0; 64],
    };
    let mut buf: [uint8_t; 1000] = [0; 1000];
    let mut sha256sum: [uint8_t; 32] = [0; 32];
    let mut hexit: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    static sha256_self_tested: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    static failures: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if sha256_self_tested.get() {
        return failures.get() as ::core::ffi::c_int == false_0;
    }
    sha256_self_tested.set(true_0 != 0);
    let mut i: size_t = 0 as size_t;
    while i < 3 as size_t {
        if i < 2 as size_t {
            hexit = sha256_bytes(
                (*sha_self_test_msg.ptr())[i as usize] as *mut uint8_t,
                strlen((*sha_self_test_msg.ptr())[i as usize]),
                ::core::ptr::null::<uint8_t>(),
                0 as size_t,
            );
            strcpy(
                &raw mut output as *mut ::core::ffi::c_char,
                hexit as *mut ::core::ffi::c_char,
            );
        } else {
            sha256_start(&raw mut ctx);
            memset(
                &raw mut buf as *mut uint8_t as *mut ::core::ffi::c_void,
                'a' as ::core::ffi::c_int,
                1000 as size_t,
            );
            let mut j: size_t = 0 as size_t;
            while j < 1000 as size_t {
                sha256_update(&raw mut ctx, &raw mut buf as *mut uint8_t, 1000 as size_t);
                j = j.wrapping_add(1);
            }
            sha256_finish(&raw mut ctx, &raw mut sha256sum as *mut uint8_t);
            let mut j_0: size_t = 0 as size_t;
            while j_0 < SHA256_SUM_SIZE as size_t {
                snprintf(
                    (&raw mut output as *mut ::core::ffi::c_char)
                        .offset(j_0.wrapping_mul(SHA_STEP as size_t) as isize),
                    (SHA_STEP + 1 as ::core::ffi::c_int) as size_t,
                    b"%02x\0".as_ptr() as *const ::core::ffi::c_char,
                    sha256sum[j_0 as usize] as ::core::ffi::c_int,
                );
                j_0 = j_0.wrapping_add(1);
            }
        }
        if memcmp(
            &raw mut output as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            (*sha_self_test_vector.ptr())[i as usize] as *const ::core::ffi::c_void,
            SHA256_BUFFER_SIZE as size_t,
        ) != 0 as ::core::ffi::c_int
        {
            failures.set(true_0 != 0);
            output[::core::mem::size_of::<[::core::ffi::c_char; 65]>().wrapping_sub(1 as usize)
                as usize] = NUL as ::core::ffi::c_char;
        }
        i = i.wrapping_add(1);
    }
    return failures.get() as ::core::ffi::c_int == false_0;
}
