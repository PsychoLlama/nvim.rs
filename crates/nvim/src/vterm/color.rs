//! The colours a terminal pen can name.
//!
//! An indexed colour resolves through a 256-entry palette. Its first sixteen
//! slots are the ANSI colours, which the host may repaint at runtime, so they
//! live in the terminal state; everything above them is fixed — a 6x6x6 RGB
//! cube followed by 24 greys.
//!
//! The C-facing `VTermColor` union stays out of this module. Everything here
//! is expressed as a [`ColorValue`], the payload that union's type byte
//! selects.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_long;

/// What a colour actually names.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ColorValue {
    /// A literal 24-bit colour.
    Rgb { red: u8, green: u8, blue: u8 },
    /// A slot in the terminal's 256-entry palette.
    Indexed(u8),
}

impl ColorValue {
    /// A literal colour.
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        ColorValue::Rgb { red, green, blue }
    }

    /// A literal grey of the given level.
    const fn grey(level: u8) -> Self {
        ColorValue::rgb(level, level, level)
    }
}

/// The ANSI colours a terminal starts with: eight base colours, then their
/// high-intensity counterparts. The base white is 0xe0 rather than 0xff so
/// that high-intensity white still reads as brighter.
const ANSI_DEFAULTS: [ColorValue; 16] = [
    ColorValue::rgb(0, 0, 0),       // black
    ColorValue::rgb(224, 0, 0),     // red
    ColorValue::rgb(0, 224, 0),     // green
    ColorValue::rgb(224, 224, 0),   // yellow
    ColorValue::rgb(0, 0, 224),     // blue
    ColorValue::rgb(224, 0, 224),   // magenta
    ColorValue::rgb(0, 224, 224),   // cyan
    ColorValue::rgb(224, 224, 224), // white, i.e. light grey
    ColorValue::rgb(128, 128, 128), // bright black
    ColorValue::rgb(255, 64, 64),   // bright red
    ColorValue::rgb(64, 255, 64),   // bright green
    ColorValue::rgb(255, 255, 64),  // bright yellow
    ColorValue::rgb(64, 64, 255),   // bright blue
    ColorValue::rgb(255, 64, 255),  // bright magenta
    ColorValue::rgb(64, 255, 255),  // bright cyan
    ColorValue::rgb(255, 255, 255), // bright white, for real
];

/// The six levels one axis of the 216-colour cube steps through.
const CUBE_LEVELS: [u8; 6] = [0x00, 0x33, 0x66, 0x99, 0xcc, 0xff];

/// The 24 greys that follow the colour cube. Neither end is pure black or
/// pure white — those are reachable through the cube.
const GREYS: [u8; 24] = [
    0x00, 0x0b, 0x16, 0x21, 0x2c, 0x37, 0x42, 0x4d, 0x58, 0x63, 0x6e, 0x79, 0x85, 0x90, 0x9b, 0xa6,
    0xb1, 0xbc, 0xc7, 0xd2, 0xdd, 0xe8, 0xf3, 0xff,
];

/// The colour ANSI palette slot `index` holds before the host repaints it,
/// or `None` outside the sixteen ANSI slots.
pub fn ansi_default(index: c_long) -> Option<ColorValue> {
    usize::try_from(index)
        .ok()
        .and_then(|i| ANSI_DEFAULTS.get(i))
        .copied()
}

/// The colour of palette slot `index`, for the part of the palette the host
/// cannot repaint: the 6x6x6 cube at 16-231 and the grey ramp at 232-255.
/// Slots 0-15 live in the terminal state, so they are not answered here.
pub fn fixed_palette(index: c_long) -> Option<ColorValue> {
    match index {
        16..=231 => {
            let cube = usize::try_from(index - 16).expect("the arm bounds it to 0..=215");
            Some(ColorValue::Rgb {
                red: CUBE_LEVELS[cube / 36],
                green: CUBE_LEVELS[cube / 6 % 6],
                blue: CUBE_LEVELS[cube % 6],
            })
        }
        232..=255 => {
            let grey = usize::try_from(index - 232).expect("the arm bounds it to 0..=23");
            Some(ColorValue::grey(GREYS[grey]))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi_slots_are_the_sixteen_defaults() {
        assert_eq!(ansi_default(0), Some(ColorValue::rgb(0, 0, 0)));
        assert_eq!(ansi_default(7), Some(ColorValue::rgb(224, 224, 224)));
        assert_eq!(ansi_default(15), Some(ColorValue::rgb(255, 255, 255)));
        assert_eq!(ansi_default(16), None);
        assert_eq!(ansi_default(-1), None);
    }

    #[test]
    fn ansi_slots_are_not_part_of_the_fixed_palette() {
        for index in -1..16 {
            assert_eq!(fixed_palette(index), None, "slot {index}");
        }
    }

    #[test]
    fn cube_steps_blue_fastest_and_red_slowest() {
        assert_eq!(fixed_palette(16), Some(ColorValue::rgb(0, 0, 0)));
        assert_eq!(fixed_palette(17), Some(ColorValue::rgb(0, 0, 0x33)));
        assert_eq!(fixed_palette(22), Some(ColorValue::rgb(0, 0x33, 0)));
        assert_eq!(fixed_palette(52), Some(ColorValue::rgb(0x33, 0, 0)));
        assert_eq!(fixed_palette(231), Some(ColorValue::rgb(0xff, 0xff, 0xff)));
    }

    #[test]
    fn grey_ramp_spans_the_last_24_slots() {
        assert_eq!(fixed_palette(232), Some(ColorValue::rgb(0, 0, 0)));
        assert_eq!(fixed_palette(243), Some(ColorValue::rgb(0x79, 0x79, 0x79)));
        assert_eq!(fixed_palette(255), Some(ColorValue::rgb(0xff, 0xff, 0xff)));
        assert_eq!(fixed_palette(256), None);
    }

    #[test]
    fn every_slot_from_16_to_255_resolves() {
        for index in 16..256 {
            assert!(fixed_palette(index).is_some(), "slot {index}");
        }
    }
}
