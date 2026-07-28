//! `[[=a=]]`: the equivalence class each base character expands to.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn reg_equi_class(mut c: ::core::ffi::c_int) {
    match c {
        65 | 192 | 193 | 194 | 195 | 196 | 197 | 256 | 258 | 260 | 461 | 478 | 480 | 506 | 514
        | 550 | 570 | 7680 | 7840 | 7842 | 7844 | 7846 | 7848 | 7850 | 7852 | 7854 | 7856
        | 7858 | 7860 | 7862 => {
            regmbc('A' as ::core::ffi::c_int);
            regmbc(0xc0 as ::core::ffi::c_int);
            regmbc(0xc1 as ::core::ffi::c_int);
            regmbc(0xc2 as ::core::ffi::c_int);
            regmbc(0xc3 as ::core::ffi::c_int);
            regmbc(0xc4 as ::core::ffi::c_int);
            regmbc(0xc5 as ::core::ffi::c_int);
            regmbc(0x100 as ::core::ffi::c_int);
            regmbc(0x102 as ::core::ffi::c_int);
            regmbc(0x104 as ::core::ffi::c_int);
            regmbc(0x1cd as ::core::ffi::c_int);
            regmbc(0x1de as ::core::ffi::c_int);
            regmbc(0x1e0 as ::core::ffi::c_int);
            regmbc(0x1fa as ::core::ffi::c_int);
            regmbc(0x202 as ::core::ffi::c_int);
            regmbc(0x226 as ::core::ffi::c_int);
            regmbc(0x23a as ::core::ffi::c_int);
            regmbc(0x1e00 as ::core::ffi::c_int);
            regmbc(0x1ea0 as ::core::ffi::c_int);
            regmbc(0x1ea2 as ::core::ffi::c_int);
            regmbc(0x1ea4 as ::core::ffi::c_int);
            regmbc(0x1ea6 as ::core::ffi::c_int);
            regmbc(0x1ea8 as ::core::ffi::c_int);
            regmbc(0x1eaa as ::core::ffi::c_int);
            regmbc(0x1eac as ::core::ffi::c_int);
            regmbc(0x1eae as ::core::ffi::c_int);
            regmbc(0x1eb0 as ::core::ffi::c_int);
            regmbc(0x1eb2 as ::core::ffi::c_int);
            regmbc(0x1eb4 as ::core::ffi::c_int);
            regmbc(0x1eb6 as ::core::ffi::c_int);
            return;
        }
        66 | 385 | 579 | 7682 | 7684 | 7686 => {
            regmbc('B' as ::core::ffi::c_int);
            regmbc(0x181 as ::core::ffi::c_int);
            regmbc(0x243 as ::core::ffi::c_int);
            regmbc(0x1e02 as ::core::ffi::c_int);
            regmbc(0x1e04 as ::core::ffi::c_int);
            regmbc(0x1e06 as ::core::ffi::c_int);
            return;
        }
        67 | 199 | 262 | 264 | 266 | 268 | 391 | 571 | 7688 | 42898 => {
            regmbc('C' as ::core::ffi::c_int);
            regmbc(0xc7 as ::core::ffi::c_int);
            regmbc(0x106 as ::core::ffi::c_int);
            regmbc(0x108 as ::core::ffi::c_int);
            regmbc(0x10a as ::core::ffi::c_int);
            regmbc(0x10c as ::core::ffi::c_int);
            regmbc(0x187 as ::core::ffi::c_int);
            regmbc(0x23b as ::core::ffi::c_int);
            regmbc(0x1e08 as ::core::ffi::c_int);
            regmbc(0xa792 as ::core::ffi::c_int);
            return;
        }
        68 | 270 | 272 | 394 | 7690 | 7692 | 7694 | 7696 | 7698 => {
            regmbc('D' as ::core::ffi::c_int);
            regmbc(0x10e as ::core::ffi::c_int);
            regmbc(0x110 as ::core::ffi::c_int);
            regmbc(0x18a as ::core::ffi::c_int);
            regmbc(0x1e0a as ::core::ffi::c_int);
            regmbc(0x1e0c as ::core::ffi::c_int);
            regmbc(0x1e0e as ::core::ffi::c_int);
            regmbc(0x1e10 as ::core::ffi::c_int);
            regmbc(0x1e12 as ::core::ffi::c_int);
            return;
        }
        69 | 200 | 201 | 202 | 203 | 274 | 276 | 278 | 280 | 282 | 516 | 518 | 552 | 582 | 7700
        | 7702 | 7704 | 7706 | 7708 | 7864 | 7866 | 7868 | 7870 | 7872 | 7874 | 7876 | 7878 => {
            regmbc('E' as ::core::ffi::c_int);
            regmbc(0xc8 as ::core::ffi::c_int);
            regmbc(0xc9 as ::core::ffi::c_int);
            regmbc(0xca as ::core::ffi::c_int);
            regmbc(0xcb as ::core::ffi::c_int);
            regmbc(0x112 as ::core::ffi::c_int);
            regmbc(0x114 as ::core::ffi::c_int);
            regmbc(0x116 as ::core::ffi::c_int);
            regmbc(0x118 as ::core::ffi::c_int);
            regmbc(0x11a as ::core::ffi::c_int);
            regmbc(0x204 as ::core::ffi::c_int);
            regmbc(0x206 as ::core::ffi::c_int);
            regmbc(0x228 as ::core::ffi::c_int);
            regmbc(0x246 as ::core::ffi::c_int);
            regmbc(0x1e14 as ::core::ffi::c_int);
            regmbc(0x1e16 as ::core::ffi::c_int);
            regmbc(0x1e18 as ::core::ffi::c_int);
            regmbc(0x1e1a as ::core::ffi::c_int);
            regmbc(0x1e1c as ::core::ffi::c_int);
            regmbc(0x1eb8 as ::core::ffi::c_int);
            regmbc(0x1eba as ::core::ffi::c_int);
            regmbc(0x1ebc as ::core::ffi::c_int);
            regmbc(0x1ebe as ::core::ffi::c_int);
            regmbc(0x1ec0 as ::core::ffi::c_int);
            regmbc(0x1ec2 as ::core::ffi::c_int);
            regmbc(0x1ec4 as ::core::ffi::c_int);
            regmbc(0x1ec6 as ::core::ffi::c_int);
            return;
        }
        70 | 401 | 7710 | 42904 => {
            regmbc('F' as ::core::ffi::c_int);
            regmbc(0x191 as ::core::ffi::c_int);
            regmbc(0x1e1e as ::core::ffi::c_int);
            regmbc(0xa798 as ::core::ffi::c_int);
            return;
        }
        71 | 284 | 286 | 288 | 290 | 403 | 484 | 486 | 500 | 7712 | 42912 => {
            regmbc('G' as ::core::ffi::c_int);
            regmbc(0x11c as ::core::ffi::c_int);
            regmbc(0x11e as ::core::ffi::c_int);
            regmbc(0x120 as ::core::ffi::c_int);
            regmbc(0x122 as ::core::ffi::c_int);
            regmbc(0x193 as ::core::ffi::c_int);
            regmbc(0x1e4 as ::core::ffi::c_int);
            regmbc(0x1e6 as ::core::ffi::c_int);
            regmbc(0x1f4 as ::core::ffi::c_int);
            regmbc(0x1e20 as ::core::ffi::c_int);
            regmbc(0xa7a0 as ::core::ffi::c_int);
            return;
        }
        72 | 292 | 294 | 542 | 7714 | 7716 | 7718 | 7720 | 7722 | 11367 => {
            regmbc('H' as ::core::ffi::c_int);
            regmbc(0x124 as ::core::ffi::c_int);
            regmbc(0x126 as ::core::ffi::c_int);
            regmbc(0x21e as ::core::ffi::c_int);
            regmbc(0x1e22 as ::core::ffi::c_int);
            regmbc(0x1e24 as ::core::ffi::c_int);
            regmbc(0x1e26 as ::core::ffi::c_int);
            regmbc(0x1e28 as ::core::ffi::c_int);
            regmbc(0x1e2a as ::core::ffi::c_int);
            regmbc(0x2c67 as ::core::ffi::c_int);
            return;
        }
        73 | 204 | 205 | 206 | 207 | 296 | 298 | 300 | 302 | 304 | 407 | 463 | 520 | 522 | 7724
        | 7726 | 7880 | 7882 => {
            regmbc('I' as ::core::ffi::c_int);
            regmbc(0xcc as ::core::ffi::c_int);
            regmbc(0xcd as ::core::ffi::c_int);
            regmbc(0xce as ::core::ffi::c_int);
            regmbc(0xcf as ::core::ffi::c_int);
            regmbc(0x128 as ::core::ffi::c_int);
            regmbc(0x12a as ::core::ffi::c_int);
            regmbc(0x12c as ::core::ffi::c_int);
            regmbc(0x12e as ::core::ffi::c_int);
            regmbc(0x130 as ::core::ffi::c_int);
            regmbc(0x197 as ::core::ffi::c_int);
            regmbc(0x1cf as ::core::ffi::c_int);
            regmbc(0x208 as ::core::ffi::c_int);
            regmbc(0x20a as ::core::ffi::c_int);
            regmbc(0x1e2c as ::core::ffi::c_int);
            regmbc(0x1e2e as ::core::ffi::c_int);
            regmbc(0x1ec8 as ::core::ffi::c_int);
            regmbc(0x1eca as ::core::ffi::c_int);
            return;
        }
        74 | 308 | 584 => {
            regmbc('J' as ::core::ffi::c_int);
            regmbc(0x134 as ::core::ffi::c_int);
            regmbc(0x248 as ::core::ffi::c_int);
            return;
        }
        75 | 310 | 408 | 488 | 7728 | 7730 | 7732 | 11369 | 42816 => {
            regmbc('K' as ::core::ffi::c_int);
            regmbc(0x136 as ::core::ffi::c_int);
            regmbc(0x198 as ::core::ffi::c_int);
            regmbc(0x1e8 as ::core::ffi::c_int);
            regmbc(0x1e30 as ::core::ffi::c_int);
            regmbc(0x1e32 as ::core::ffi::c_int);
            regmbc(0x1e34 as ::core::ffi::c_int);
            regmbc(0x2c69 as ::core::ffi::c_int);
            regmbc(0xa740 as ::core::ffi::c_int);
            return;
        }
        76 | 313 | 315 | 317 | 319 | 321 | 573 | 7734 | 7736 | 7738 | 7740 | 11360 => {
            regmbc('L' as ::core::ffi::c_int);
            regmbc(0x139 as ::core::ffi::c_int);
            regmbc(0x13b as ::core::ffi::c_int);
            regmbc(0x13d as ::core::ffi::c_int);
            regmbc(0x13f as ::core::ffi::c_int);
            regmbc(0x141 as ::core::ffi::c_int);
            regmbc(0x23d as ::core::ffi::c_int);
            regmbc(0x1e36 as ::core::ffi::c_int);
            regmbc(0x1e38 as ::core::ffi::c_int);
            regmbc(0x1e3a as ::core::ffi::c_int);
            regmbc(0x1e3c as ::core::ffi::c_int);
            regmbc(0x2c60 as ::core::ffi::c_int);
            return;
        }
        77 | 7742 | 7744 | 7746 => {
            regmbc('M' as ::core::ffi::c_int);
            regmbc(0x1e3e as ::core::ffi::c_int);
            regmbc(0x1e40 as ::core::ffi::c_int);
            regmbc(0x1e42 as ::core::ffi::c_int);
            return;
        }
        78 | 209 | 323 | 325 | 327 | 504 | 7748 | 7750 | 7752 | 7754 | 42916 => {
            regmbc('N' as ::core::ffi::c_int);
            regmbc(0xd1 as ::core::ffi::c_int);
            regmbc(0x143 as ::core::ffi::c_int);
            regmbc(0x145 as ::core::ffi::c_int);
            regmbc(0x147 as ::core::ffi::c_int);
            regmbc(0x1f8 as ::core::ffi::c_int);
            regmbc(0x1e44 as ::core::ffi::c_int);
            regmbc(0x1e46 as ::core::ffi::c_int);
            regmbc(0x1e48 as ::core::ffi::c_int);
            regmbc(0x1e4a as ::core::ffi::c_int);
            regmbc(0xa7a4 as ::core::ffi::c_int);
            return;
        }
        79 | 210 | 211 | 212 | 213 | 214 | 216 | 332 | 334 | 336 | 415 | 416 | 465 | 490 | 492
        | 510 | 524 | 526 | 554 | 556 | 558 | 560 | 7756 | 7758 | 7760 | 7762 | 7884 | 7886
        | 7888 | 7890 | 7892 | 7894 | 7896 | 7898 | 7900 | 7902 | 7904 | 7906 => {
            regmbc('O' as ::core::ffi::c_int);
            regmbc(0xd2 as ::core::ffi::c_int);
            regmbc(0xd3 as ::core::ffi::c_int);
            regmbc(0xd4 as ::core::ffi::c_int);
            regmbc(0xd5 as ::core::ffi::c_int);
            regmbc(0xd6 as ::core::ffi::c_int);
            regmbc(0xd8 as ::core::ffi::c_int);
            regmbc(0x14c as ::core::ffi::c_int);
            regmbc(0x14e as ::core::ffi::c_int);
            regmbc(0x150 as ::core::ffi::c_int);
            regmbc(0x19f as ::core::ffi::c_int);
            regmbc(0x1a0 as ::core::ffi::c_int);
            regmbc(0x1d1 as ::core::ffi::c_int);
            regmbc(0x1ea as ::core::ffi::c_int);
            regmbc(0x1ec as ::core::ffi::c_int);
            regmbc(0x1fe as ::core::ffi::c_int);
            regmbc(0x20c as ::core::ffi::c_int);
            regmbc(0x20e as ::core::ffi::c_int);
            regmbc(0x22a as ::core::ffi::c_int);
            regmbc(0x22c as ::core::ffi::c_int);
            regmbc(0x22e as ::core::ffi::c_int);
            regmbc(0x230 as ::core::ffi::c_int);
            regmbc(0x1e4c as ::core::ffi::c_int);
            regmbc(0x1e4e as ::core::ffi::c_int);
            regmbc(0x1e50 as ::core::ffi::c_int);
            regmbc(0x1e52 as ::core::ffi::c_int);
            regmbc(0x1ecc as ::core::ffi::c_int);
            regmbc(0x1ece as ::core::ffi::c_int);
            regmbc(0x1ed0 as ::core::ffi::c_int);
            regmbc(0x1ed2 as ::core::ffi::c_int);
            regmbc(0x1ed4 as ::core::ffi::c_int);
            regmbc(0x1ed6 as ::core::ffi::c_int);
            regmbc(0x1ed8 as ::core::ffi::c_int);
            regmbc(0x1eda as ::core::ffi::c_int);
            regmbc(0x1edc as ::core::ffi::c_int);
            regmbc(0x1ede as ::core::ffi::c_int);
            regmbc(0x1ee0 as ::core::ffi::c_int);
            regmbc(0x1ee2 as ::core::ffi::c_int);
            return;
        }
        80 | 420 | 7764 | 7766 | 11363 => {
            regmbc('P' as ::core::ffi::c_int);
            regmbc(0x1a4 as ::core::ffi::c_int);
            regmbc(0x1e54 as ::core::ffi::c_int);
            regmbc(0x1e56 as ::core::ffi::c_int);
            regmbc(0x2c63 as ::core::ffi::c_int);
            return;
        }
        81 | 586 => {
            regmbc('Q' as ::core::ffi::c_int);
            regmbc(0x24a as ::core::ffi::c_int);
            return;
        }
        82 | 340 | 342 | 344 | 528 | 530 | 588 | 7768 | 7770 | 7772 | 7774 | 11364 | 42918 => {
            regmbc('R' as ::core::ffi::c_int);
            regmbc(0x154 as ::core::ffi::c_int);
            regmbc(0x156 as ::core::ffi::c_int);
            regmbc(0x210 as ::core::ffi::c_int);
            regmbc(0x212 as ::core::ffi::c_int);
            regmbc(0x158 as ::core::ffi::c_int);
            regmbc(0x24c as ::core::ffi::c_int);
            regmbc(0x1e58 as ::core::ffi::c_int);
            regmbc(0x1e5a as ::core::ffi::c_int);
            regmbc(0x1e5c as ::core::ffi::c_int);
            regmbc(0x1e5e as ::core::ffi::c_int);
            regmbc(0x2c64 as ::core::ffi::c_int);
            regmbc(0xa7a6 as ::core::ffi::c_int);
            return;
        }
        83 | 346 | 348 | 350 | 352 | 536 | 7776 | 7778 | 7780 | 7782 | 7784 | 11390 | 42920 => {
            regmbc('S' as ::core::ffi::c_int);
            regmbc(0x15a as ::core::ffi::c_int);
            regmbc(0x15c as ::core::ffi::c_int);
            regmbc(0x15e as ::core::ffi::c_int);
            regmbc(0x160 as ::core::ffi::c_int);
            regmbc(0x218 as ::core::ffi::c_int);
            regmbc(0x1e60 as ::core::ffi::c_int);
            regmbc(0x1e62 as ::core::ffi::c_int);
            regmbc(0x1e64 as ::core::ffi::c_int);
            regmbc(0x1e66 as ::core::ffi::c_int);
            regmbc(0x1e68 as ::core::ffi::c_int);
            regmbc(0x2c7e as ::core::ffi::c_int);
            regmbc(0xa7a8 as ::core::ffi::c_int);
            return;
        }
        84 | 354 | 356 | 358 | 428 | 430 | 538 | 574 | 7786 | 7788 | 7790 | 7792 => {
            regmbc('T' as ::core::ffi::c_int);
            regmbc(0x162 as ::core::ffi::c_int);
            regmbc(0x164 as ::core::ffi::c_int);
            regmbc(0x166 as ::core::ffi::c_int);
            regmbc(0x1ac as ::core::ffi::c_int);
            regmbc(0x23e as ::core::ffi::c_int);
            regmbc(0x1ae as ::core::ffi::c_int);
            regmbc(0x21a as ::core::ffi::c_int);
            regmbc(0x1e6a as ::core::ffi::c_int);
            regmbc(0x1e6c as ::core::ffi::c_int);
            regmbc(0x1e6e as ::core::ffi::c_int);
            regmbc(0x1e70 as ::core::ffi::c_int);
            return;
        }
        85 | 217 | 218 | 219 | 220 | 360 | 362 | 364 | 366 | 368 | 370 | 431 | 467 | 469 | 471
        | 473 | 475 | 532 | 534 | 580 | 7794 | 7796 | 7798 | 7800 | 7802 | 7908 | 7910 | 7912
        | 7914 | 7916 | 7918 | 7920 => {
            regmbc('U' as ::core::ffi::c_int);
            regmbc(0xd9 as ::core::ffi::c_int);
            regmbc(0xda as ::core::ffi::c_int);
            regmbc(0xdb as ::core::ffi::c_int);
            regmbc(0xdc as ::core::ffi::c_int);
            regmbc(0x168 as ::core::ffi::c_int);
            regmbc(0x16a as ::core::ffi::c_int);
            regmbc(0x16c as ::core::ffi::c_int);
            regmbc(0x16e as ::core::ffi::c_int);
            regmbc(0x170 as ::core::ffi::c_int);
            regmbc(0x172 as ::core::ffi::c_int);
            regmbc(0x1af as ::core::ffi::c_int);
            regmbc(0x1d3 as ::core::ffi::c_int);
            regmbc(0x1d5 as ::core::ffi::c_int);
            regmbc(0x1d7 as ::core::ffi::c_int);
            regmbc(0x1d9 as ::core::ffi::c_int);
            regmbc(0x1db as ::core::ffi::c_int);
            regmbc(0x214 as ::core::ffi::c_int);
            regmbc(0x216 as ::core::ffi::c_int);
            regmbc(0x244 as ::core::ffi::c_int);
            regmbc(0x1e72 as ::core::ffi::c_int);
            regmbc(0x1e74 as ::core::ffi::c_int);
            regmbc(0x1e76 as ::core::ffi::c_int);
            regmbc(0x1e78 as ::core::ffi::c_int);
            regmbc(0x1e7a as ::core::ffi::c_int);
            regmbc(0x1ee4 as ::core::ffi::c_int);
            regmbc(0x1ee6 as ::core::ffi::c_int);
            regmbc(0x1ee8 as ::core::ffi::c_int);
            regmbc(0x1eea as ::core::ffi::c_int);
            regmbc(0x1eec as ::core::ffi::c_int);
            regmbc(0x1eee as ::core::ffi::c_int);
            regmbc(0x1ef0 as ::core::ffi::c_int);
            return;
        }
        86 | 434 | 7804 | 7806 => {
            regmbc('V' as ::core::ffi::c_int);
            regmbc(0x1b2 as ::core::ffi::c_int);
            regmbc(0x1e7c as ::core::ffi::c_int);
            regmbc(0x1e7e as ::core::ffi::c_int);
            return;
        }
        87 | 372 | 7808 | 7810 | 7812 | 7814 | 7816 => {
            regmbc('W' as ::core::ffi::c_int);
            regmbc(0x174 as ::core::ffi::c_int);
            regmbc(0x1e80 as ::core::ffi::c_int);
            regmbc(0x1e82 as ::core::ffi::c_int);
            regmbc(0x1e84 as ::core::ffi::c_int);
            regmbc(0x1e86 as ::core::ffi::c_int);
            regmbc(0x1e88 as ::core::ffi::c_int);
            return;
        }
        88 | 7818 | 7820 => {
            regmbc('X' as ::core::ffi::c_int);
            regmbc(0x1e8a as ::core::ffi::c_int);
            regmbc(0x1e8c as ::core::ffi::c_int);
            return;
        }
        89 | 221 | 374 | 376 | 435 | 562 | 590 | 7822 | 7922 | 7926 | 7924 | 7928 => {
            regmbc('Y' as ::core::ffi::c_int);
            regmbc(0xdd as ::core::ffi::c_int);
            regmbc(0x176 as ::core::ffi::c_int);
            regmbc(0x178 as ::core::ffi::c_int);
            regmbc(0x1b3 as ::core::ffi::c_int);
            regmbc(0x232 as ::core::ffi::c_int);
            regmbc(0x24e as ::core::ffi::c_int);
            regmbc(0x1e8e as ::core::ffi::c_int);
            regmbc(0x1ef2 as ::core::ffi::c_int);
            regmbc(0x1ef4 as ::core::ffi::c_int);
            regmbc(0x1ef6 as ::core::ffi::c_int);
            regmbc(0x1ef8 as ::core::ffi::c_int);
            return;
        }
        90 | 377 | 379 | 381 | 437 | 7824 | 7826 | 7828 | 11371 => {
            regmbc('Z' as ::core::ffi::c_int);
            regmbc(0x179 as ::core::ffi::c_int);
            regmbc(0x17b as ::core::ffi::c_int);
            regmbc(0x17d as ::core::ffi::c_int);
            regmbc(0x1b5 as ::core::ffi::c_int);
            regmbc(0x1e90 as ::core::ffi::c_int);
            regmbc(0x1e92 as ::core::ffi::c_int);
            regmbc(0x1e94 as ::core::ffi::c_int);
            regmbc(0x2c6b as ::core::ffi::c_int);
            return;
        }
        97 | 224 | 225 | 226 | 227 | 228 | 229 | 257 | 259 | 261 | 462 | 479 | 481 | 507 | 513
        | 515 | 551 | 7567 | 7681 | 7834 | 7841 | 7843 | 7845 | 7847 | 7849 | 7851 | 7853
        | 7855 | 7857 | 7859 | 7861 | 7863 | 11365 => {
            regmbc('a' as ::core::ffi::c_int);
            regmbc(0xe0 as ::core::ffi::c_int);
            regmbc(0xe1 as ::core::ffi::c_int);
            regmbc(0xe2 as ::core::ffi::c_int);
            regmbc(0xe3 as ::core::ffi::c_int);
            regmbc(0xe4 as ::core::ffi::c_int);
            regmbc(0xe5 as ::core::ffi::c_int);
            regmbc(0x101 as ::core::ffi::c_int);
            regmbc(0x103 as ::core::ffi::c_int);
            regmbc(0x105 as ::core::ffi::c_int);
            regmbc(0x1ce as ::core::ffi::c_int);
            regmbc(0x1df as ::core::ffi::c_int);
            regmbc(0x1e1 as ::core::ffi::c_int);
            regmbc(0x1fb as ::core::ffi::c_int);
            regmbc(0x201 as ::core::ffi::c_int);
            regmbc(0x203 as ::core::ffi::c_int);
            regmbc(0x227 as ::core::ffi::c_int);
            regmbc(0x1d8f as ::core::ffi::c_int);
            regmbc(0x1e01 as ::core::ffi::c_int);
            regmbc(0x1e9a as ::core::ffi::c_int);
            regmbc(0x1ea1 as ::core::ffi::c_int);
            regmbc(0x1ea3 as ::core::ffi::c_int);
            regmbc(0x1ea5 as ::core::ffi::c_int);
            regmbc(0x1ea7 as ::core::ffi::c_int);
            regmbc(0x1ea9 as ::core::ffi::c_int);
            regmbc(0x1eab as ::core::ffi::c_int);
            regmbc(0x1ead as ::core::ffi::c_int);
            regmbc(0x1eaf as ::core::ffi::c_int);
            regmbc(0x1eb1 as ::core::ffi::c_int);
            regmbc(0x1eb3 as ::core::ffi::c_int);
            regmbc(0x1eb5 as ::core::ffi::c_int);
            regmbc(0x1eb7 as ::core::ffi::c_int);
            regmbc(0x2c65 as ::core::ffi::c_int);
            return;
        }
        98 | 384 | 595 | 7532 | 7552 | 7683 | 7685 | 7687 => {
            regmbc('b' as ::core::ffi::c_int);
            regmbc(0x180 as ::core::ffi::c_int);
            regmbc(0x253 as ::core::ffi::c_int);
            regmbc(0x1d6c as ::core::ffi::c_int);
            regmbc(0x1d80 as ::core::ffi::c_int);
            regmbc(0x1e03 as ::core::ffi::c_int);
            regmbc(0x1e05 as ::core::ffi::c_int);
            regmbc(0x1e07 as ::core::ffi::c_int);
            return;
        }
        99 | 231 | 263 | 265 | 267 | 269 | 392 | 572 | 7689 | 42899 | 42900 => {
            regmbc('c' as ::core::ffi::c_int);
            regmbc(0xe7 as ::core::ffi::c_int);
            regmbc(0x107 as ::core::ffi::c_int);
            regmbc(0x109 as ::core::ffi::c_int);
            regmbc(0x10b as ::core::ffi::c_int);
            regmbc(0x10d as ::core::ffi::c_int);
            regmbc(0x188 as ::core::ffi::c_int);
            regmbc(0x23c as ::core::ffi::c_int);
            regmbc(0x1e09 as ::core::ffi::c_int);
            regmbc(0xa793 as ::core::ffi::c_int);
            regmbc(0xa794 as ::core::ffi::c_int);
            return;
        }
        100 | 271 | 273 | 599 | 7533 | 7553 | 7569 | 7691 | 7693 | 7695 | 7697 | 7699 => {
            regmbc('d' as ::core::ffi::c_int);
            regmbc(0x10f as ::core::ffi::c_int);
            regmbc(0x111 as ::core::ffi::c_int);
            regmbc(0x257 as ::core::ffi::c_int);
            regmbc(0x1d6d as ::core::ffi::c_int);
            regmbc(0x1d81 as ::core::ffi::c_int);
            regmbc(0x1d91 as ::core::ffi::c_int);
            regmbc(0x1e0b as ::core::ffi::c_int);
            regmbc(0x1e0d as ::core::ffi::c_int);
            regmbc(0x1e0f as ::core::ffi::c_int);
            regmbc(0x1e11 as ::core::ffi::c_int);
            regmbc(0x1e13 as ::core::ffi::c_int);
            return;
        }
        101 | 232 | 233 | 234 | 235 | 275 | 277 | 279 | 281 | 283 | 517 | 519 | 553 | 583
        | 7570 | 7701 | 7703 | 7705 | 7707 | 7865 | 7867 | 7709 | 7869 | 7871 | 7873 | 7875
        | 7877 | 7879 => {
            regmbc('e' as ::core::ffi::c_int);
            regmbc(0xe8 as ::core::ffi::c_int);
            regmbc(0xe9 as ::core::ffi::c_int);
            regmbc(0xea as ::core::ffi::c_int);
            regmbc(0xeb as ::core::ffi::c_int);
            regmbc(0x113 as ::core::ffi::c_int);
            regmbc(0x115 as ::core::ffi::c_int);
            regmbc(0x117 as ::core::ffi::c_int);
            regmbc(0x119 as ::core::ffi::c_int);
            regmbc(0x11b as ::core::ffi::c_int);
            regmbc(0x205 as ::core::ffi::c_int);
            regmbc(0x207 as ::core::ffi::c_int);
            regmbc(0x229 as ::core::ffi::c_int);
            regmbc(0x247 as ::core::ffi::c_int);
            regmbc(0x1d92 as ::core::ffi::c_int);
            regmbc(0x1e15 as ::core::ffi::c_int);
            regmbc(0x1e17 as ::core::ffi::c_int);
            regmbc(0x1e19 as ::core::ffi::c_int);
            regmbc(0x1e1b as ::core::ffi::c_int);
            regmbc(0x1e1d as ::core::ffi::c_int);
            regmbc(0x1eb9 as ::core::ffi::c_int);
            regmbc(0x1ebb as ::core::ffi::c_int);
            regmbc(0x1ebd as ::core::ffi::c_int);
            regmbc(0x1ebf as ::core::ffi::c_int);
            regmbc(0x1ec1 as ::core::ffi::c_int);
            regmbc(0x1ec3 as ::core::ffi::c_int);
            regmbc(0x1ec5 as ::core::ffi::c_int);
            regmbc(0x1ec7 as ::core::ffi::c_int);
            return;
        }
        102 | 402 | 7534 | 7554 | 7711 | 42905 => {
            regmbc('f' as ::core::ffi::c_int);
            regmbc(0x192 as ::core::ffi::c_int);
            regmbc(0x1d6e as ::core::ffi::c_int);
            regmbc(0x1d82 as ::core::ffi::c_int);
            regmbc(0x1e1f as ::core::ffi::c_int);
            regmbc(0xa799 as ::core::ffi::c_int);
            return;
        }
        103 | 285 | 287 | 289 | 291 | 485 | 487 | 608 | 501 | 7555 | 7713 | 42913 => {
            regmbc('g' as ::core::ffi::c_int);
            regmbc(0x11d as ::core::ffi::c_int);
            regmbc(0x11f as ::core::ffi::c_int);
            regmbc(0x121 as ::core::ffi::c_int);
            regmbc(0x123 as ::core::ffi::c_int);
            regmbc(0x1e5 as ::core::ffi::c_int);
            regmbc(0x1e7 as ::core::ffi::c_int);
            regmbc(0x1f5 as ::core::ffi::c_int);
            regmbc(0x260 as ::core::ffi::c_int);
            regmbc(0x1d83 as ::core::ffi::c_int);
            regmbc(0x1e21 as ::core::ffi::c_int);
            regmbc(0xa7a1 as ::core::ffi::c_int);
            return;
        }
        104 | 293 | 295 | 543 | 7715 | 7717 | 7719 | 7721 | 7723 | 7830 | 11368 | 42901 => {
            regmbc('h' as ::core::ffi::c_int);
            regmbc(0x125 as ::core::ffi::c_int);
            regmbc(0x127 as ::core::ffi::c_int);
            regmbc(0x21f as ::core::ffi::c_int);
            regmbc(0x1e23 as ::core::ffi::c_int);
            regmbc(0x1e25 as ::core::ffi::c_int);
            regmbc(0x1e27 as ::core::ffi::c_int);
            regmbc(0x1e29 as ::core::ffi::c_int);
            regmbc(0x1e2b as ::core::ffi::c_int);
            regmbc(0x1e96 as ::core::ffi::c_int);
            regmbc(0x2c68 as ::core::ffi::c_int);
            regmbc(0xa795 as ::core::ffi::c_int);
            return;
        }
        105 | 236 | 237 | 238 | 239 | 297 | 299 | 301 | 303 | 464 | 521 | 523 | 616 | 7574
        | 7725 | 7727 | 7881 | 7883 => {
            regmbc('i' as ::core::ffi::c_int);
            regmbc(0xec as ::core::ffi::c_int);
            regmbc(0xed as ::core::ffi::c_int);
            regmbc(0xee as ::core::ffi::c_int);
            regmbc(0xef as ::core::ffi::c_int);
            regmbc(0x129 as ::core::ffi::c_int);
            regmbc(0x12b as ::core::ffi::c_int);
            regmbc(0x12d as ::core::ffi::c_int);
            regmbc(0x12f as ::core::ffi::c_int);
            regmbc(0x1d0 as ::core::ffi::c_int);
            regmbc(0x209 as ::core::ffi::c_int);
            regmbc(0x20b as ::core::ffi::c_int);
            regmbc(0x268 as ::core::ffi::c_int);
            regmbc(0x1d96 as ::core::ffi::c_int);
            regmbc(0x1e2d as ::core::ffi::c_int);
            regmbc(0x1e2f as ::core::ffi::c_int);
            regmbc(0x1ec9 as ::core::ffi::c_int);
            regmbc(0x1ecb as ::core::ffi::c_int);
            return;
        }
        106 | 309 | 496 | 585 => {
            regmbc('j' as ::core::ffi::c_int);
            regmbc(0x135 as ::core::ffi::c_int);
            regmbc(0x1f0 as ::core::ffi::c_int);
            regmbc(0x249 as ::core::ffi::c_int);
            return;
        }
        107 | 311 | 409 | 489 | 7556 | 7729 | 7731 | 7733 | 11370 | 42817 => {
            regmbc('k' as ::core::ffi::c_int);
            regmbc(0x137 as ::core::ffi::c_int);
            regmbc(0x199 as ::core::ffi::c_int);
            regmbc(0x1e9 as ::core::ffi::c_int);
            regmbc(0x1d84 as ::core::ffi::c_int);
            regmbc(0x1e31 as ::core::ffi::c_int);
            regmbc(0x1e33 as ::core::ffi::c_int);
            regmbc(0x1e35 as ::core::ffi::c_int);
            regmbc(0x2c6a as ::core::ffi::c_int);
            regmbc(0xa741 as ::core::ffi::c_int);
            return;
        }
        108 | 314 | 316 | 318 | 320 | 322 | 410 | 7735 | 7737 | 7739 | 7741 | 11361 => {
            regmbc('l' as ::core::ffi::c_int);
            regmbc(0x13a as ::core::ffi::c_int);
            regmbc(0x13c as ::core::ffi::c_int);
            regmbc(0x13e as ::core::ffi::c_int);
            regmbc(0x140 as ::core::ffi::c_int);
            regmbc(0x142 as ::core::ffi::c_int);
            regmbc(0x19a as ::core::ffi::c_int);
            regmbc(0x1e37 as ::core::ffi::c_int);
            regmbc(0x1e39 as ::core::ffi::c_int);
            regmbc(0x1e3b as ::core::ffi::c_int);
            regmbc(0x1e3d as ::core::ffi::c_int);
            regmbc(0x2c61 as ::core::ffi::c_int);
            return;
        }
        109 | 7535 | 7743 | 7745 | 7747 => {
            regmbc('m' as ::core::ffi::c_int);
            regmbc(0x1d6f as ::core::ffi::c_int);
            regmbc(0x1e3f as ::core::ffi::c_int);
            regmbc(0x1e41 as ::core::ffi::c_int);
            regmbc(0x1e43 as ::core::ffi::c_int);
            return;
        }
        110 | 241 | 324 | 326 | 328 | 329 | 505 | 7536 | 7559 | 7749 | 7751 | 7753 | 7755
        | 42917 => {
            regmbc('n' as ::core::ffi::c_int);
            regmbc(0xf1 as ::core::ffi::c_int);
            regmbc(0x144 as ::core::ffi::c_int);
            regmbc(0x146 as ::core::ffi::c_int);
            regmbc(0x148 as ::core::ffi::c_int);
            regmbc(0x149 as ::core::ffi::c_int);
            regmbc(0x1f9 as ::core::ffi::c_int);
            regmbc(0x1d70 as ::core::ffi::c_int);
            regmbc(0x1d87 as ::core::ffi::c_int);
            regmbc(0x1e45 as ::core::ffi::c_int);
            regmbc(0x1e47 as ::core::ffi::c_int);
            regmbc(0x1e49 as ::core::ffi::c_int);
            regmbc(0x1e4b as ::core::ffi::c_int);
            regmbc(0xa7a5 as ::core::ffi::c_int);
            return;
        }
        111 | 242 | 243 | 244 | 245 | 246 | 248 | 333 | 335 | 337 | 417 | 466 | 491 | 493 | 511
        | 525 | 527 | 555 | 557 | 559 | 561 | 629 | 7757 | 7759 | 7761 | 7763 | 7885 | 7887
        | 7889 | 7891 | 7893 | 7895 | 7897 | 7899 | 7901 | 7903 | 7905 | 7907 => {
            regmbc('o' as ::core::ffi::c_int);
            regmbc(0xf2 as ::core::ffi::c_int);
            regmbc(0xf3 as ::core::ffi::c_int);
            regmbc(0xf4 as ::core::ffi::c_int);
            regmbc(0xf5 as ::core::ffi::c_int);
            regmbc(0xf6 as ::core::ffi::c_int);
            regmbc(0xf8 as ::core::ffi::c_int);
            regmbc(0x14d as ::core::ffi::c_int);
            regmbc(0x14f as ::core::ffi::c_int);
            regmbc(0x151 as ::core::ffi::c_int);
            regmbc(0x1a1 as ::core::ffi::c_int);
            regmbc(0x1d2 as ::core::ffi::c_int);
            regmbc(0x1eb as ::core::ffi::c_int);
            regmbc(0x1ed as ::core::ffi::c_int);
            regmbc(0x1ff as ::core::ffi::c_int);
            regmbc(0x20d as ::core::ffi::c_int);
            regmbc(0x20f as ::core::ffi::c_int);
            regmbc(0x22b as ::core::ffi::c_int);
            regmbc(0x22d as ::core::ffi::c_int);
            regmbc(0x22f as ::core::ffi::c_int);
            regmbc(0x231 as ::core::ffi::c_int);
            regmbc(0x275 as ::core::ffi::c_int);
            regmbc(0x1e4d as ::core::ffi::c_int);
            regmbc(0x1e4f as ::core::ffi::c_int);
            regmbc(0x1e51 as ::core::ffi::c_int);
            regmbc(0x1e53 as ::core::ffi::c_int);
            regmbc(0x1ecd as ::core::ffi::c_int);
            regmbc(0x1ecf as ::core::ffi::c_int);
            regmbc(0x1ed1 as ::core::ffi::c_int);
            regmbc(0x1ed3 as ::core::ffi::c_int);
            regmbc(0x1ed5 as ::core::ffi::c_int);
            regmbc(0x1ed7 as ::core::ffi::c_int);
            regmbc(0x1ed9 as ::core::ffi::c_int);
            regmbc(0x1edb as ::core::ffi::c_int);
            regmbc(0x1edd as ::core::ffi::c_int);
            regmbc(0x1edf as ::core::ffi::c_int);
            regmbc(0x1ee1 as ::core::ffi::c_int);
            regmbc(0x1ee3 as ::core::ffi::c_int);
            return;
        }
        112 | 421 | 7537 | 7560 | 7549 | 7765 | 7767 => {
            regmbc('p' as ::core::ffi::c_int);
            regmbc(0x1a5 as ::core::ffi::c_int);
            regmbc(0x1d71 as ::core::ffi::c_int);
            regmbc(0x1d7d as ::core::ffi::c_int);
            regmbc(0x1d88 as ::core::ffi::c_int);
            regmbc(0x1e55 as ::core::ffi::c_int);
            regmbc(0x1e57 as ::core::ffi::c_int);
            return;
        }
        113 | 587 | 672 => {
            regmbc('q' as ::core::ffi::c_int);
            regmbc(0x24b as ::core::ffi::c_int);
            regmbc(0x2a0 as ::core::ffi::c_int);
            return;
        }
        114 | 341 | 343 | 345 | 529 | 531 | 589 | 637 | 7538 | 7539 | 7561 | 7769 | 7771 | 7773
        | 7775 | 42919 => {
            regmbc('r' as ::core::ffi::c_int);
            regmbc(0x155 as ::core::ffi::c_int);
            regmbc(0x157 as ::core::ffi::c_int);
            regmbc(0x159 as ::core::ffi::c_int);
            regmbc(0x211 as ::core::ffi::c_int);
            regmbc(0x213 as ::core::ffi::c_int);
            regmbc(0x24d as ::core::ffi::c_int);
            regmbc(0x1d72 as ::core::ffi::c_int);
            regmbc(0x1d73 as ::core::ffi::c_int);
            regmbc(0x1d89 as ::core::ffi::c_int);
            regmbc(0x1e59 as ::core::ffi::c_int);
            regmbc(0x27d as ::core::ffi::c_int);
            regmbc(0x1e5b as ::core::ffi::c_int);
            regmbc(0x1e5d as ::core::ffi::c_int);
            regmbc(0x1e5f as ::core::ffi::c_int);
            regmbc(0xa7a7 as ::core::ffi::c_int);
            return;
        }
        115 | 347 | 349 | 351 | 353 | 7777 | 537 | 575 | 7540 | 7562 | 7779 | 7781 | 7783
        | 7785 | 42921 => {
            regmbc('s' as ::core::ffi::c_int);
            regmbc(0x15b as ::core::ffi::c_int);
            regmbc(0x15d as ::core::ffi::c_int);
            regmbc(0x15f as ::core::ffi::c_int);
            regmbc(0x161 as ::core::ffi::c_int);
            regmbc(0x23f as ::core::ffi::c_int);
            regmbc(0x219 as ::core::ffi::c_int);
            regmbc(0x1d74 as ::core::ffi::c_int);
            regmbc(0x1d8a as ::core::ffi::c_int);
            regmbc(0x1e61 as ::core::ffi::c_int);
            regmbc(0x1e63 as ::core::ffi::c_int);
            regmbc(0x1e65 as ::core::ffi::c_int);
            regmbc(0x1e67 as ::core::ffi::c_int);
            regmbc(0x1e69 as ::core::ffi::c_int);
            regmbc(0xa7a9 as ::core::ffi::c_int);
            return;
        }
        116 | 355 | 357 | 359 | 427 | 429 | 539 | 648 | 7541 | 7787 | 7789 | 7791 | 7793 | 7831
        | 11366 => {
            regmbc('t' as ::core::ffi::c_int);
            regmbc(0x163 as ::core::ffi::c_int);
            regmbc(0x165 as ::core::ffi::c_int);
            regmbc(0x167 as ::core::ffi::c_int);
            regmbc(0x1ab as ::core::ffi::c_int);
            regmbc(0x21b as ::core::ffi::c_int);
            regmbc(0x1ad as ::core::ffi::c_int);
            regmbc(0x288 as ::core::ffi::c_int);
            regmbc(0x1d75 as ::core::ffi::c_int);
            regmbc(0x1e6b as ::core::ffi::c_int);
            regmbc(0x1e6d as ::core::ffi::c_int);
            regmbc(0x1e6f as ::core::ffi::c_int);
            regmbc(0x1e71 as ::core::ffi::c_int);
            regmbc(0x1e97 as ::core::ffi::c_int);
            regmbc(0x2c66 as ::core::ffi::c_int);
            return;
        }
        117 | 249 | 250 | 251 | 252 | 361 | 363 | 365 | 367 | 369 | 371 | 432 | 468 | 470 | 472
        | 474 | 476 | 533 | 535 | 649 | 7795 | 7550 | 7577 | 7797 | 7799 | 7801 | 7803 | 7909
        | 7911 | 7913 | 7915 | 7917 | 7919 | 7921 => {
            regmbc('u' as ::core::ffi::c_int);
            regmbc(0xf9 as ::core::ffi::c_int);
            regmbc(0xfa as ::core::ffi::c_int);
            regmbc(0xfb as ::core::ffi::c_int);
            regmbc(0xfc as ::core::ffi::c_int);
            regmbc(0x169 as ::core::ffi::c_int);
            regmbc(0x16b as ::core::ffi::c_int);
            regmbc(0x16d as ::core::ffi::c_int);
            regmbc(0x16f as ::core::ffi::c_int);
            regmbc(0x171 as ::core::ffi::c_int);
            regmbc(0x173 as ::core::ffi::c_int);
            regmbc(0x1d6 as ::core::ffi::c_int);
            regmbc(0x1d8 as ::core::ffi::c_int);
            regmbc(0x1da as ::core::ffi::c_int);
            regmbc(0x1dc as ::core::ffi::c_int);
            regmbc(0x215 as ::core::ffi::c_int);
            regmbc(0x217 as ::core::ffi::c_int);
            regmbc(0x1b0 as ::core::ffi::c_int);
            regmbc(0x1d4 as ::core::ffi::c_int);
            regmbc(0x289 as ::core::ffi::c_int);
            regmbc(0x1d7e as ::core::ffi::c_int);
            regmbc(0x1d99 as ::core::ffi::c_int);
            regmbc(0x1e73 as ::core::ffi::c_int);
            regmbc(0x1e75 as ::core::ffi::c_int);
            regmbc(0x1e77 as ::core::ffi::c_int);
            regmbc(0x1e79 as ::core::ffi::c_int);
            regmbc(0x1e7b as ::core::ffi::c_int);
            regmbc(0x1ee5 as ::core::ffi::c_int);
            regmbc(0x1ee7 as ::core::ffi::c_int);
            regmbc(0x1ee9 as ::core::ffi::c_int);
            regmbc(0x1eeb as ::core::ffi::c_int);
            regmbc(0x1eed as ::core::ffi::c_int);
            regmbc(0x1eef as ::core::ffi::c_int);
            regmbc(0x1ef1 as ::core::ffi::c_int);
            return;
        }
        118 | 651 | 7564 | 7805 | 7807 => {
            regmbc('v' as ::core::ffi::c_int);
            regmbc(0x28b as ::core::ffi::c_int);
            regmbc(0x1d8c as ::core::ffi::c_int);
            regmbc(0x1e7d as ::core::ffi::c_int);
            regmbc(0x1e7f as ::core::ffi::c_int);
            return;
        }
        119 | 373 | 7809 | 7811 | 7813 | 7815 | 7817 | 7832 => {
            regmbc('w' as ::core::ffi::c_int);
            regmbc(0x175 as ::core::ffi::c_int);
            regmbc(0x1e81 as ::core::ffi::c_int);
            regmbc(0x1e83 as ::core::ffi::c_int);
            regmbc(0x1e85 as ::core::ffi::c_int);
            regmbc(0x1e87 as ::core::ffi::c_int);
            regmbc(0x1e89 as ::core::ffi::c_int);
            regmbc(0x1e98 as ::core::ffi::c_int);
            return;
        }
        120 | 7819 | 7821 => {
            regmbc('x' as ::core::ffi::c_int);
            regmbc(0x1e8b as ::core::ffi::c_int);
            regmbc(0x1e8d as ::core::ffi::c_int);
            return;
        }
        121 | 253 | 255 | 375 | 436 | 563 | 591 | 7823 | 7833 | 7923 | 7925 | 7927 | 7929 => {
            regmbc('y' as ::core::ffi::c_int);
            regmbc(0xfd as ::core::ffi::c_int);
            regmbc(0xff as ::core::ffi::c_int);
            regmbc(0x177 as ::core::ffi::c_int);
            regmbc(0x1b4 as ::core::ffi::c_int);
            regmbc(0x233 as ::core::ffi::c_int);
            regmbc(0x24f as ::core::ffi::c_int);
            regmbc(0x1e8f as ::core::ffi::c_int);
            regmbc(0x1e99 as ::core::ffi::c_int);
            regmbc(0x1ef3 as ::core::ffi::c_int);
            regmbc(0x1ef5 as ::core::ffi::c_int);
            regmbc(0x1ef7 as ::core::ffi::c_int);
            regmbc(0x1ef9 as ::core::ffi::c_int);
            return;
        }
        122 | 378 | 380 | 382 | 438 | 7542 | 7566 | 7825 | 7827 | 7829 | 11372 => {
            regmbc('z' as ::core::ffi::c_int);
            regmbc(0x17a as ::core::ffi::c_int);
            regmbc(0x17c as ::core::ffi::c_int);
            regmbc(0x17e as ::core::ffi::c_int);
            regmbc(0x1b6 as ::core::ffi::c_int);
            regmbc(0x1d76 as ::core::ffi::c_int);
            regmbc(0x1d8e as ::core::ffi::c_int);
            regmbc(0x1e91 as ::core::ffi::c_int);
            regmbc(0x1e93 as ::core::ffi::c_int);
            regmbc(0x1e95 as ::core::ffi::c_int);
            regmbc(0x2c6c as ::core::ffi::c_int);
            return;
        }
        _ => {}
    }
    regmbc(c);
}
