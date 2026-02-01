//! Unicode equivalence classes for case-insensitive regex matching.
//!
//! This module provides equivalence class data used by both the BT (backtracking)
//! and NFA regex engines for case-insensitive matching with `\c` or `ignorecase`.
//!
//! For example, 'A' is equivalent to À, Á, Â, Ã, Ä, Å, and many other accented forms.

use std::ffi::c_int;

use crate::nfa_states::NFA_CONCAT;

// FFI declarations for emitting
extern "C" {
    fn nvim_regmbc(c: c_int);
    fn nvim_nfa_emit(c: c_int);
}

/// Equivalence class for uppercase 'A' and variants
const CLASS_A_UPPER: &[u32] = &[
    'A' as u32, 0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0x100, 0x102, 0x104, 0x1cd, 0x1de, 0x1e0,
    0x1fa, 0x200, 0x202, 0x226, 0x23a, 0x1e00, 0x1ea0, 0x1ea2, 0x1ea4, 0x1ea6, 0x1ea8, 0x1eaa,
    0x1eac, 0x1eae, 0x1eb0, 0x1eb2, 0x1eb4, 0x1eb6,
];

/// Equivalence class for uppercase 'B' and variants
const CLASS_B_UPPER: &[u32] = &['B' as u32, 0x181, 0x243, 0x1e02, 0x1e04, 0x1e06];

/// Equivalence class for uppercase 'C' and variants
const CLASS_C_UPPER: &[u32] = &[
    'C' as u32, 0xc7, 0x106, 0x108, 0x10a, 0x10c, 0x187, 0x23b, 0x1e08, 0xa792,
];

/// Equivalence class for uppercase 'D' and variants
const CLASS_D_UPPER: &[u32] = &[
    'D' as u32, 0x10e, 0x110, 0x18a, 0x1e0a, 0x1e0c, 0x1e0e, 0x1e10, 0x1e12,
];

/// Equivalence class for uppercase 'E' and variants
const CLASS_E_UPPER: &[u32] = &[
    'E' as u32, 0xc8, 0xc9, 0xca, 0xcb, 0x112, 0x114, 0x116, 0x118, 0x11a, 0x204, 0x206, 0x228,
    0x246, 0x1e14, 0x1e16, 0x1e18, 0x1e1a, 0x1e1c, 0x1eb8, 0x1eba, 0x1ebc, 0x1ebe, 0x1ec0, 0x1ec2,
    0x1ec4, 0x1ec6,
];

/// Equivalence class for uppercase 'F' and variants
const CLASS_F_UPPER: &[u32] = &['F' as u32, 0x191, 0x1e1e, 0xa798];

/// Equivalence class for uppercase 'G' and variants
const CLASS_G_UPPER: &[u32] = &[
    'G' as u32, 0x11c, 0x11e, 0x120, 0x122, 0x193, 0x1e4, 0x1e6, 0x1f4, 0x1e20, 0xa7a0,
];

/// Equivalence class for uppercase 'H' and variants
const CLASS_H_UPPER: &[u32] = &[
    'H' as u32, 0x124, 0x126, 0x21e, 0x1e22, 0x1e24, 0x1e26, 0x1e28, 0x1e2a, 0x2c67,
];

/// Equivalence class for uppercase 'I' and variants
const CLASS_I_UPPER: &[u32] = &[
    'I' as u32, 0xcc, 0xcd, 0xce, 0xcf, 0x128, 0x12a, 0x12c, 0x12e, 0x130, 0x197, 0x1cf, 0x208,
    0x20a, 0x1e2c, 0x1e2e, 0x1ec8, 0x1eca,
];

/// Equivalence class for uppercase 'J' and variants
const CLASS_J_UPPER: &[u32] = &['J' as u32, 0x134, 0x248];

/// Equivalence class for uppercase 'K' and variants
const CLASS_K_UPPER: &[u32] = &[
    'K' as u32, 0x136, 0x198, 0x1e8, 0x1e30, 0x1e32, 0x1e34, 0x2c69, 0xa740,
];

/// Equivalence class for uppercase 'L' and variants
const CLASS_L_UPPER: &[u32] = &[
    'L' as u32, 0x139, 0x13b, 0x13d, 0x13f, 0x141, 0x23d, 0x1e36, 0x1e38, 0x1e3a, 0x1e3c, 0x2c60,
];

/// Equivalence class for uppercase 'M' and variants
const CLASS_M_UPPER: &[u32] = &['M' as u32, 0x1e3e, 0x1e40, 0x1e42];

/// Equivalence class for uppercase 'N' and variants
const CLASS_N_UPPER: &[u32] = &[
    'N' as u32, 0xd1, 0x143, 0x145, 0x147, 0x1f8, 0x1e44, 0x1e46, 0x1e48, 0x1e4a, 0xa7a4,
];

/// Equivalence class for uppercase 'O' and variants
const CLASS_O_UPPER: &[u32] = &[
    'O' as u32, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd8, 0x14c, 0x14e, 0x150, 0x19f, 0x1a0, 0x1d1,
    0x1ea, 0x1ec, 0x1fe, 0x20c, 0x20e, 0x22a, 0x22c, 0x22e, 0x230, 0x1e4c, 0x1e4e, 0x1e50, 0x1e52,
    0x1ecc, 0x1ece, 0x1ed0, 0x1ed2, 0x1ed4, 0x1ed6, 0x1ed8, 0x1eda, 0x1edc, 0x1ede, 0x1ee0, 0x1ee2,
];

/// Equivalence class for uppercase 'P' and variants
const CLASS_P_UPPER: &[u32] = &['P' as u32, 0x1a4, 0x1e54, 0x1e56, 0x2c63];

/// Equivalence class for uppercase 'Q' and variants
const CLASS_Q_UPPER: &[u32] = &['Q' as u32, 0x24a];

/// Equivalence class for uppercase 'R' and variants
const CLASS_R_UPPER: &[u32] = &[
    'R' as u32, 0x154, 0x156, 0x158, 0x210, 0x212, 0x24c, 0x1e58, 0x1e5a, 0x1e5c, 0x1e5e, 0x2c64,
    0xa7a6,
];

/// Equivalence class for uppercase 'S' and variants
const CLASS_S_UPPER: &[u32] = &[
    'S' as u32, 0x15a, 0x15c, 0x15e, 0x160, 0x218, 0x1e60, 0x1e62, 0x1e64, 0x1e66, 0x1e68, 0x2c7e,
    0xa7a8,
];

/// Equivalence class for uppercase 'T' and variants
const CLASS_T_UPPER: &[u32] = &[
    'T' as u32, 0x162, 0x164, 0x166, 0x1ac, 0x1ae, 0x21a, 0x23e, 0x1e6a, 0x1e6c, 0x1e6e, 0x1e70,
];

/// Equivalence class for uppercase 'U' and variants
const CLASS_U_UPPER: &[u32] = &[
    'U' as u32, 0xd9, 0xda, 0xdb, 0xdc, 0x168, 0x16a, 0x16c, 0x16e, 0x170, 0x172, 0x1af, 0x1d3,
    0x1d5, 0x1d7, 0x1d9, 0x1db, 0x214, 0x216, 0x244, 0x1e72, 0x1e74, 0x1e76, 0x1e78, 0x1e7a,
    0x1ee4, 0x1ee6, 0x1ee8, 0x1eea, 0x1eec, 0x1eee, 0x1ef0,
];

/// Equivalence class for uppercase 'V' and variants
const CLASS_V_UPPER: &[u32] = &['V' as u32, 0x1b2, 0x1e7c, 0x1e7e];

/// Equivalence class for uppercase 'W' and variants
const CLASS_W_UPPER: &[u32] = &['W' as u32, 0x174, 0x1e80, 0x1e82, 0x1e84, 0x1e86, 0x1e88];

/// Equivalence class for uppercase 'X' and variants
const CLASS_X_UPPER: &[u32] = &['X' as u32, 0x1e8a, 0x1e8c];

/// Equivalence class for uppercase 'Y' and variants
const CLASS_Y_UPPER: &[u32] = &[
    'Y' as u32, 0xdd, 0x176, 0x178, 0x1b3, 0x232, 0x24e, 0x1e8e, 0x1ef2, 0x1ef4, 0x1ef6, 0x1ef8,
];

/// Equivalence class for uppercase 'Z' and variants
const CLASS_Z_UPPER: &[u32] = &[
    'Z' as u32, 0x179, 0x17b, 0x17d, 0x1b5, 0x1e90, 0x1e92, 0x1e94, 0x2c6b,
];

/// Equivalence class for lowercase 'a' and variants
const CLASS_A_LOWER: &[u32] = &[
    'a' as u32, 0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0x101, 0x103, 0x105, 0x1ce, 0x1df, 0x1e1,
    0x1fb, 0x201, 0x203, 0x227, 0x1d8f, 0x1e01, 0x1e9a, 0x1ea1, 0x1ea3, 0x1ea5, 0x1ea7, 0x1ea9,
    0x1eab, 0x1ead, 0x1eaf, 0x1eb1, 0x1eb3, 0x1eb5, 0x1eb7, 0x2c65,
];

/// Equivalence class for lowercase 'b' and variants
const CLASS_B_LOWER: &[u32] = &[
    'b' as u32, 0x180, 0x253, 0x1d6c, 0x1d80, 0x1e03, 0x1e05, 0x1e07,
];

/// Equivalence class for lowercase 'c' and variants
const CLASS_C_LOWER: &[u32] = &[
    'c' as u32, 0xe7, 0x107, 0x109, 0x10b, 0x10d, 0x188, 0x23c, 0x1e09, 0xa793, 0xa794,
];

/// Equivalence class for lowercase 'd' and variants
const CLASS_D_LOWER: &[u32] = &[
    'd' as u32, 0x10f, 0x111, 0x257, 0x1d6d, 0x1d81, 0x1d91, 0x1e0b, 0x1e0d, 0x1e0f, 0x1e11, 0x1e13,
];

/// Equivalence class for lowercase 'e' and variants
const CLASS_E_LOWER: &[u32] = &[
    'e' as u32, 0xe8, 0xe9, 0xea, 0xeb, 0x113, 0x115, 0x117, 0x119, 0x11b, 0x205, 0x207, 0x229,
    0x247, 0x1d92, 0x1e15, 0x1e17, 0x1e19, 0x1e1b, 0x1e1d, 0x1eb9, 0x1ebb, 0x1ebd, 0x1ebf, 0x1ec1,
    0x1ec3, 0x1ec5, 0x1ec7,
];

/// Equivalence class for lowercase 'f' and variants
const CLASS_F_LOWER: &[u32] = &['f' as u32, 0x192, 0x1d6e, 0x1d82, 0x1e1f, 0xa799];

/// Equivalence class for lowercase 'g' and variants
const CLASS_G_LOWER: &[u32] = &[
    'g' as u32, 0x11d, 0x11f, 0x121, 0x123, 0x1e5, 0x1e7, 0x1f5, 0x260, 0x1d83, 0x1e21, 0xa7a1,
];

/// Equivalence class for lowercase 'h' and variants
const CLASS_H_LOWER: &[u32] = &[
    'h' as u32, 0x125, 0x127, 0x21f, 0x1e23, 0x1e25, 0x1e27, 0x1e29, 0x1e2b, 0x1e96, 0x2c68, 0xa795,
];

/// Equivalence class for lowercase 'i' and variants
const CLASS_I_LOWER: &[u32] = &[
    'i' as u32, 0xec, 0xed, 0xee, 0xef, 0x129, 0x12b, 0x12d, 0x12f, 0x1d0, 0x209, 0x20b, 0x268,
    0x1d96, 0x1e2d, 0x1e2f, 0x1ec9, 0x1ecb,
];

/// Equivalence class for lowercase 'j' and variants
const CLASS_J_LOWER: &[u32] = &['j' as u32, 0x135, 0x1f0, 0x249];

/// Equivalence class for lowercase 'k' and variants
const CLASS_K_LOWER: &[u32] = &[
    'k' as u32, 0x137, 0x199, 0x1e9, 0x1d84, 0x1e31, 0x1e33, 0x1e35, 0x2c6a, 0xa741,
];

/// Equivalence class for lowercase 'l' and variants
const CLASS_L_LOWER: &[u32] = &[
    'l' as u32, 0x13a, 0x13c, 0x13e, 0x140, 0x142, 0x19a, 0x1e37, 0x1e39, 0x1e3b, 0x1e3d, 0x2c61,
];

/// Equivalence class for lowercase 'm' and variants
const CLASS_M_LOWER: &[u32] = &['m' as u32, 0x1d6f, 0x1e3f, 0x1e41, 0x1e43];

/// Equivalence class for lowercase 'n' and variants
const CLASS_N_LOWER: &[u32] = &[
    'n' as u32, 0xf1, 0x144, 0x146, 0x148, 0x149, 0x1f9, 0x1d70, 0x1d87, 0x1e45, 0x1e47, 0x1e49,
    0x1e4b, 0xa7a5,
];

/// Equivalence class for lowercase 'o' and variants
const CLASS_O_LOWER: &[u32] = &[
    'o' as u32, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf8, 0x14d, 0x14f, 0x151, 0x1a1, 0x1d2, 0x1eb,
    0x1ed, 0x1ff, 0x20d, 0x20f, 0x22b, 0x22d, 0x22f, 0x231, 0x275, 0x1e4d, 0x1e4f, 0x1e51, 0x1e53,
    0x1ecd, 0x1ecf, 0x1ed1, 0x1ed3, 0x1ed5, 0x1ed7, 0x1ed9, 0x1edb, 0x1edd, 0x1edf, 0x1ee1, 0x1ee3,
];

/// Equivalence class for lowercase 'p' and variants
const CLASS_P_LOWER: &[u32] = &['p' as u32, 0x1a5, 0x1d71, 0x1d7d, 0x1d88, 0x1e55, 0x1e57];

/// Equivalence class for lowercase 'q' and variants
const CLASS_Q_LOWER: &[u32] = &['q' as u32, 0x24b, 0x2a0];

/// Equivalence class for lowercase 'r' and variants
const CLASS_R_LOWER: &[u32] = &[
    'r' as u32, 0x155, 0x157, 0x159, 0x211, 0x213, 0x24d, 0x27d, 0x1d72, 0x1d73, 0x1d89, 0x1e59,
    0x1e5b, 0x1e5d, 0x1e5f, 0xa7a7,
];

/// Equivalence class for lowercase 's' and variants
const CLASS_S_LOWER: &[u32] = &[
    's' as u32, 0x15b, 0x15d, 0x15f, 0x161, 0x219, 0x23f, 0x1d74, 0x1d8a, 0x1e61, 0x1e63, 0x1e65,
    0x1e67, 0x1e69, 0xa7a9,
];

/// Equivalence class for lowercase 't' and variants
const CLASS_T_LOWER: &[u32] = &[
    't' as u32, 0x163, 0x165, 0x167, 0x1ab, 0x1ad, 0x21b, 0x288, 0x1d75, 0x1e6b, 0x1e6d, 0x1e6f,
    0x1e71, 0x1e97, 0x2c66,
];

/// Equivalence class for lowercase 'u' and variants
const CLASS_U_LOWER: &[u32] = &[
    'u' as u32, 0xf9, 0xfa, 0xfb, 0xfc, 0x169, 0x16b, 0x16d, 0x16f, 0x171, 0x173, 0x1b0, 0x1d4,
    0x1d6, 0x1d8, 0x1da, 0x1dc, 0x215, 0x217, 0x289, 0x1d7e, 0x1d99, 0x1e73, 0x1e75, 0x1e77,
    0x1e79, 0x1e7b, 0x1ee5, 0x1ee7, 0x1ee9, 0x1eeb, 0x1eed, 0x1eef, 0x1ef1,
];

/// Equivalence class for lowercase 'v' and variants
const CLASS_V_LOWER: &[u32] = &['v' as u32, 0x28b, 0x1d8c, 0x1e7d, 0x1e7f];

/// Equivalence class for lowercase 'w' and variants
const CLASS_W_LOWER: &[u32] = &[
    'w' as u32, 0x175, 0x1e81, 0x1e83, 0x1e85, 0x1e87, 0x1e89, 0x1e98,
];

/// Equivalence class for lowercase 'x' and variants
const CLASS_X_LOWER: &[u32] = &['x' as u32, 0x1e8b, 0x1e8d];

/// Equivalence class for lowercase 'y' and variants
const CLASS_Y_LOWER: &[u32] = &[
    'y' as u32, 0xfd, 0xff, 0x177, 0x1b4, 0x233, 0x24f, 0x1e8f, 0x1e99, 0x1ef3, 0x1ef5, 0x1ef7,
    0x1ef9,
];

/// Equivalence class for lowercase 'z' and variants
const CLASS_Z_LOWER: &[u32] = &[
    'z' as u32, 0x17a, 0x17c, 0x17e, 0x1b6, 0x1d76, 0x1d8e, 0x1e91, 0x1e93, 0x1e95, 0x2c6c,
];

/// Entry in the equivalence class lookup table
struct EquiClassEntry {
    /// All characters that map to this equivalence class
    members: &'static [u32],
}

/// Lookup table mapping characters to their equivalence classes.
/// Sorted by the base character for binary search.
static EQUI_CLASSES: &[(u32, EquiClassEntry)] = &[
    (
        'A' as u32,
        EquiClassEntry {
            members: CLASS_A_UPPER,
        },
    ),
    (
        'B' as u32,
        EquiClassEntry {
            members: CLASS_B_UPPER,
        },
    ),
    (
        'C' as u32,
        EquiClassEntry {
            members: CLASS_C_UPPER,
        },
    ),
    (
        'D' as u32,
        EquiClassEntry {
            members: CLASS_D_UPPER,
        },
    ),
    (
        'E' as u32,
        EquiClassEntry {
            members: CLASS_E_UPPER,
        },
    ),
    (
        'F' as u32,
        EquiClassEntry {
            members: CLASS_F_UPPER,
        },
    ),
    (
        'G' as u32,
        EquiClassEntry {
            members: CLASS_G_UPPER,
        },
    ),
    (
        'H' as u32,
        EquiClassEntry {
            members: CLASS_H_UPPER,
        },
    ),
    (
        'I' as u32,
        EquiClassEntry {
            members: CLASS_I_UPPER,
        },
    ),
    (
        'J' as u32,
        EquiClassEntry {
            members: CLASS_J_UPPER,
        },
    ),
    (
        'K' as u32,
        EquiClassEntry {
            members: CLASS_K_UPPER,
        },
    ),
    (
        'L' as u32,
        EquiClassEntry {
            members: CLASS_L_UPPER,
        },
    ),
    (
        'M' as u32,
        EquiClassEntry {
            members: CLASS_M_UPPER,
        },
    ),
    (
        'N' as u32,
        EquiClassEntry {
            members: CLASS_N_UPPER,
        },
    ),
    (
        'O' as u32,
        EquiClassEntry {
            members: CLASS_O_UPPER,
        },
    ),
    (
        'P' as u32,
        EquiClassEntry {
            members: CLASS_P_UPPER,
        },
    ),
    (
        'Q' as u32,
        EquiClassEntry {
            members: CLASS_Q_UPPER,
        },
    ),
    (
        'R' as u32,
        EquiClassEntry {
            members: CLASS_R_UPPER,
        },
    ),
    (
        'S' as u32,
        EquiClassEntry {
            members: CLASS_S_UPPER,
        },
    ),
    (
        'T' as u32,
        EquiClassEntry {
            members: CLASS_T_UPPER,
        },
    ),
    (
        'U' as u32,
        EquiClassEntry {
            members: CLASS_U_UPPER,
        },
    ),
    (
        'V' as u32,
        EquiClassEntry {
            members: CLASS_V_UPPER,
        },
    ),
    (
        'W' as u32,
        EquiClassEntry {
            members: CLASS_W_UPPER,
        },
    ),
    (
        'X' as u32,
        EquiClassEntry {
            members: CLASS_X_UPPER,
        },
    ),
    (
        'Y' as u32,
        EquiClassEntry {
            members: CLASS_Y_UPPER,
        },
    ),
    (
        'Z' as u32,
        EquiClassEntry {
            members: CLASS_Z_UPPER,
        },
    ),
    (
        'a' as u32,
        EquiClassEntry {
            members: CLASS_A_LOWER,
        },
    ),
    (
        'b' as u32,
        EquiClassEntry {
            members: CLASS_B_LOWER,
        },
    ),
    (
        'c' as u32,
        EquiClassEntry {
            members: CLASS_C_LOWER,
        },
    ),
    (
        'd' as u32,
        EquiClassEntry {
            members: CLASS_D_LOWER,
        },
    ),
    (
        'e' as u32,
        EquiClassEntry {
            members: CLASS_E_LOWER,
        },
    ),
    (
        'f' as u32,
        EquiClassEntry {
            members: CLASS_F_LOWER,
        },
    ),
    (
        'g' as u32,
        EquiClassEntry {
            members: CLASS_G_LOWER,
        },
    ),
    (
        'h' as u32,
        EquiClassEntry {
            members: CLASS_H_LOWER,
        },
    ),
    (
        'i' as u32,
        EquiClassEntry {
            members: CLASS_I_LOWER,
        },
    ),
    (
        'j' as u32,
        EquiClassEntry {
            members: CLASS_J_LOWER,
        },
    ),
    (
        'k' as u32,
        EquiClassEntry {
            members: CLASS_K_LOWER,
        },
    ),
    (
        'l' as u32,
        EquiClassEntry {
            members: CLASS_L_LOWER,
        },
    ),
    (
        'm' as u32,
        EquiClassEntry {
            members: CLASS_M_LOWER,
        },
    ),
    (
        'n' as u32,
        EquiClassEntry {
            members: CLASS_N_LOWER,
        },
    ),
    (
        'o' as u32,
        EquiClassEntry {
            members: CLASS_O_LOWER,
        },
    ),
    (
        'p' as u32,
        EquiClassEntry {
            members: CLASS_P_LOWER,
        },
    ),
    (
        'q' as u32,
        EquiClassEntry {
            members: CLASS_Q_LOWER,
        },
    ),
    (
        'r' as u32,
        EquiClassEntry {
            members: CLASS_R_LOWER,
        },
    ),
    (
        's' as u32,
        EquiClassEntry {
            members: CLASS_S_LOWER,
        },
    ),
    (
        't' as u32,
        EquiClassEntry {
            members: CLASS_T_LOWER,
        },
    ),
    (
        'u' as u32,
        EquiClassEntry {
            members: CLASS_U_LOWER,
        },
    ),
    (
        'v' as u32,
        EquiClassEntry {
            members: CLASS_V_LOWER,
        },
    ),
    (
        'w' as u32,
        EquiClassEntry {
            members: CLASS_W_LOWER,
        },
    ),
    (
        'x' as u32,
        EquiClassEntry {
            members: CLASS_X_LOWER,
        },
    ),
    (
        'y' as u32,
        EquiClassEntry {
            members: CLASS_Y_LOWER,
        },
    ),
    (
        'z' as u32,
        EquiClassEntry {
            members: CLASS_Z_LOWER,
        },
    ),
];

/// Get the equivalence class for a character.
///
/// Returns the slice of all characters in the equivalence class, or None
/// if the character has no equivalence class (only matches itself).
fn get_equi_class(c: u32) -> Option<&'static [u32]> {
    // First check if c is a base character (A-Z, a-z)
    if let Ok(idx) = EQUI_CLASSES.binary_search_by_key(&c, |&(base, _)| base) {
        return Some(EQUI_CLASSES[idx].1.members);
    }

    // Search through all classes to find if c is a member
    for (_, entry) in EQUI_CLASSES {
        if entry.members.contains(&c) {
            return Some(entry.members);
        }
    }

    None
}

/// Emit equivalence class for BT engine using regmbc.
///
/// # Safety
/// Must be called during BT regex compilation.
#[no_mangle]
pub unsafe extern "C" fn rs_reg_equi_class(c: c_int) {
    if let Some(members) = get_equi_class(c as u32) {
        for &member in members {
            nvim_regmbc(member as c_int);
        }
    } else {
        // No equivalence class, emit character itself
        nvim_regmbc(c);
    }
}

/// Emit equivalence class for NFA engine using EMIT + NFA_CONCAT.
///
/// # Safety
/// Must be called during NFA regex compilation.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_emit_equi_class(c: c_int) {
    if let Some(members) = get_equi_class(c as u32) {
        for &member in members {
            nvim_nfa_emit(member as c_int);
            nvim_nfa_emit(NFA_CONCAT);
        }
    } else {
        // No equivalence class, emit character itself
        nvim_nfa_emit(c);
        nvim_nfa_emit(NFA_CONCAT);
    }
}

/// Emit equivalence class for NFA engine (internal Rust call).
///
/// # Safety
/// Must be called during NFA regex compilation.
pub unsafe fn emit_nfa_equi_class(c: c_int) {
    rs_nfa_emit_equi_class(c);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_equi_class_base_chars() {
        // Test base characters
        assert!(get_equi_class('A' as u32).is_some());
        assert!(get_equi_class('a' as u32).is_some());
        assert!(get_equi_class('Z' as u32).is_some());
        assert!(get_equi_class('z' as u32).is_some());
    }

    #[test]
    fn test_get_equi_class_accented() {
        // Test accented characters map to their class
        let class_a = get_equi_class(0xc0).unwrap(); // À
        assert!(class_a.contains(&('A' as u32)));
        assert!(class_a.contains(&0xc0));
        assert!(class_a.contains(&0xc1));

        let class_a_lower = get_equi_class(0xe0).unwrap(); // à
        assert!(class_a_lower.contains(&('a' as u32)));
    }

    #[test]
    fn test_no_equi_class() {
        // Characters without equivalence class
        assert!(get_equi_class('0' as u32).is_none());
        assert!(get_equi_class('!' as u32).is_none());
        assert!(get_equi_class(' ' as u32).is_none());
    }

    #[test]
    fn test_class_sizes() {
        // Verify some expected class sizes
        assert_eq!(CLASS_A_UPPER.len(), 31);
        assert_eq!(CLASS_A_LOWER.len(), 33);
        assert_eq!(CLASS_O_UPPER.len(), 38);
        assert_eq!(CLASS_O_LOWER.len(), 38);
    }
}
