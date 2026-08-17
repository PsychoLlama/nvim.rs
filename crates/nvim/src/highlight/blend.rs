#![deny(unsafe_op_in_unsafe_fn)]

//! Blending one attribute set through another: `'winblend'`, `'pumblend'`.
//!
//! A blend is a *cell* operation, not a group one — the compositor knows
//! which cell of the window below is under this cell of the float, and the
//! answer depends on both. So [`hl_blend_attrs`] takes the two ids, mixes
//! their colours by the front set's `hl_blend` percentage, and hands back an
//! attribute id for the result, memoised per pair (see [`cache`]).
//!
//! `through` is the second axis. A blended cell that is *blank* lets the
//! character below show through, which means the result keeps the back set's
//! foreground and takes only the background from the front; a non-blank one
//! keeps its own foreground. The two get separate caches because they are
//! separate answers for the same pair.
//!
//! Everything here works in forced colours: the cell arithmetic needs real
//! numbers, so an unset colour is resolved to `Normal`'s and failing that to
//! the black/white implied by `'background'`. That is also where `HL_INVERSE`
//! is applied and dropped — the terminal would otherwise invert the *blended*
//! colours, which is not what the unblended cell would have looked like.

use super::cache::AttrCache;
use super::{
    HL_BG_INDEXED, HL_FG_INDEXED, HL_INVERSE, HL_UNDERLINE_MASK, get_attr_entry, kHlBlend,
    kHlBlendThrough, syn_attr2entry, update_window_hl,
};
use crate::global_cell::GlobalCell;
use crate::highlight_group::highlight_changed;
use crate::main::{curwin, normal_bg, normal_fg, normal_sp, p_bg};
use crate::types::{HlAttrs, HlEntry, RgbValue, int16_t};
use core::ffi::c_int;

/// Blends of a blank cell over another, by `(back, front)`.
static BLEND_THROUGH: GlobalCell<AttrCache> = GlobalCell::new(AttrCache::new());

/// Blends of a non-blank cell over another, by `(back, front)`.
static BLEND: GlobalCell<AttrCache> = GlobalCell::new(AttrCache::new());

/// Drops every remembered blend. The colours they were mixed from have
/// changed, so the answers are stale even though the pairs are not.
pub fn clear_caches() {
    BLEND.with_mut(AttrCache::clear);
    BLEND_THROUGH.with_mut(AttrCache::clear);
}

/// [`clear_caches`], plus the redraw that makes it visible.
///
/// # Safety
/// Reaches the current window and the highlight tables; main thread only.
pub unsafe fn hl_invalidate_blends() {
    clear_caches();
    // SAFETY: the editor's own globals.
    unsafe {
        highlight_changed();
        update_window_hl(curwin.get(), true);
    }
}

/// The attribute set `front_attr` blended over `back_attr`.
///
/// Answers `front_attr` unchanged when there is nothing to blend: either
/// side uninitialised (a negative id), or a front set with no `blend=`
/// percentage — in which case `through` is also cleared, telling the
/// compositor the cell below is hidden after all.
///
/// # Safety
/// Reads the attribute table; main thread only.
pub unsafe fn hl_blend_attrs(back_attr: c_int, front_attr: c_int, through: &mut bool) -> c_int {
    // An uninitialised background cell has nothing to show through.
    if front_attr < 0 || back_attr < 0 {
        return front_attr;
    }
    // SAFETY: the attribute table is the editor's own.
    unsafe {
        let front_raw = syn_attr2entry(front_attr);
        let front = get_colors_force(front_raw);
        let ratio = front.hl_blend;
        if ratio <= 0 {
            *through = false;
            return front_attr;
        }

        let cache = if *through { &BLEND_THROUGH } else { &BLEND };
        let cached = (*cache.ptr()).get(back_attr, front_attr);
        if cached > 0 {
            return cached;
        }

        let back_raw = syn_attr2entry(back_attr);
        let back = get_colors_force(back_raw);
        let mut blended = if *through {
            blend_through(ratio, back, back_raw, front)
        } else {
            blend_over(ratio, back, front)
        };

        // A fully transparent background stays transparent, so that the
        // terminal's own background keeps showing. At ratio 100 the front
        // contributes nothing, so the back's transparency alone decides.
        blended.rgb_bg_color = if ratio == 100 && back_raw.rgb_bg_color == -1 {
            -1
        } else if back_raw.rgb_bg_color == -1 && front_raw.rgb_bg_color == -1 {
            -1
        } else {
            rgb_blend(ratio, back.rgb_bg_color, front.rgb_bg_color)
        };
        // The blend property was consumed producing this set.
        blended.hl_blend = -1;

        let kind = if *through { kHlBlendThrough } else { kHlBlend };
        let id = get_attr_entry(HlEntry {
            attr: blended,
            kind,
            id1: back_attr,
            id2: front_attr,
        });
        if id > 0 {
            (*cache.ptr()).insert(back_attr, front_attr, id);
        }
        id
    }
}

/// A blank front cell over `back`: the character below stays visible, so the
/// result is the back set with the front's *background* mixed into it.
fn blend_through(ratio: c_int, back: HlAttrs, back_raw: HlAttrs, front: HlAttrs) -> HlAttrs {
    let mut blended = back;
    blended.rgb_fg_color = rgb_blend(ratio, back.rgb_fg_color, front.rgb_bg_color);
    // Blend the special colour only where the cell below asked for one;
    // otherwise it is cleared rather than mixed towards the default red.
    blended.rgb_sp_color =
        if blended.rgb_ae_attr & HL_UNDERLINE_MASK != 0 && back_raw.rgb_sp_color != -1 {
            rgb_blend(ratio, back.rgb_sp_color, front.rgb_bg_color)
        } else {
            -1
        };
    blended.cterm_bg_color = front.cterm_bg_color;
    blended.cterm_fg_color = cterm_blend(ratio, back.cterm_fg_color, front.cterm_bg_color);
    // Both colours are now mixtures, so neither is a palette index.
    blended.rgb_ae_attr &= !(HL_FG_INDEXED | HL_BG_INDEXED);
    blended
}

/// A non-blank front cell over `back`: the front's own text is drawn, its
/// foreground pulled halfway towards the one it covers.
fn blend_over(ratio: c_int, back: HlAttrs, front: HlAttrs) -> HlAttrs {
    let mut blended = front;
    blended.rgb_fg_color = rgb_blend(ratio / 2, back.rgb_fg_color, front.rgb_fg_color);
    blended.rgb_sp_color = if blended.rgb_ae_attr & HL_UNDERLINE_MASK != 0 {
        rgb_blend(ratio / 2, back.rgb_bg_color, front.rgb_sp_color)
    } else {
        -1
    };
    blended.rgb_ae_attr &= !(HL_FG_INDEXED | HL_BG_INDEXED);
    blended
}

/// `attrs` with every RGB colour resolved to a real number.
///
/// Unset falls back to `Normal`'s colour and then to the black-on-white or
/// white-on-black `'background'` implies; special falls back to red. Cterm
/// colours are left alone — they have their own 0-means-unset convention and
/// [`cterm_blend`] resolves them itself.
///
/// # Safety
/// Reads the `Normal` colours and `'background'`; main thread only.
unsafe fn get_colors_force(mut attrs: HlAttrs) -> HlAttrs {
    // SAFETY: the editor's own globals; `p_bg` is a NUL-terminated option
    // string, never empty.
    let dark = unsafe { *p_bg.get() == b'd' as ::core::ffi::c_char };
    if attrs.rgb_bg_color == -1 {
        attrs.rgb_bg_color = normal_bg.get();
    }
    if attrs.rgb_fg_color == -1 {
        attrs.rgb_fg_color = normal_fg.get();
    }
    if attrs.rgb_sp_color == -1 {
        attrs.rgb_sp_color = normal_sp.get();
    }
    if attrs.rgb_fg_color == -1 {
        attrs.rgb_fg_color = if dark { 0xffffff } else { 0x000000 };
    }
    if attrs.rgb_bg_color == -1 {
        attrs.rgb_bg_color = if dark { 0x000000 } else { 0xffffff };
    }
    if attrs.rgb_sp_color == -1 {
        attrs.rgb_sp_color = 0xff0000;
    }
    // Apply inversion here rather than leaving it to the terminal: what is
    // blended has to be the colours the cell would really have shown.
    if attrs.rgb_ae_attr & HL_INVERSE != 0 {
        core::mem::swap(&mut attrs.rgb_bg_color, &mut attrs.rgb_fg_color);
        attrs.rgb_ae_attr &= !HL_INVERSE;
    }
    attrs
}

/// `ratio` percent of `rgb1` plus the rest of `rgb2`, per channel.
fn rgb_blend(ratio: c_int, rgb1: RgbValue, rgb2: RgbValue) -> RgbValue {
    let a = ratio;
    let b = 100 - ratio;
    let mix = |shift: c_int| {
        let c1 = (rgb1 >> shift) & 0xff;
        let c2 = (rgb2 >> shift) & 0xff;
        (a * c1 + b * c2) / 100
    };
    (mix(16) << 16) + (mix(8) << 8) + mix(0)
}

/// [`rgb_blend`] for two colour numbers: out to RGB, mix, back again.
fn cterm_blend(ratio: c_int, c1: int16_t, c2: int16_t) -> int16_t {
    let rgb1 = cterm2rgb(c_int::from(c1));
    let rgb2 = cterm2rgb(c_int::from(c2));
    rgb2cterm(rgb_blend(ratio, rgb1, rgb2)) as int16_t
}

/// The nearest colour-cube number to an RGB colour. Only the 216-colour cube
/// is considered, so the answer is always in 0..216.
fn rgb2cterm(rgb: RgbValue) -> c_int {
    let level = |shift: c_int| ((rgb >> shift) & 0xff) * 6 / 256;
    level(16) * 36 + level(8) * 6 + level(0)
}

/// An xterm colour number as RGB.
fn cterm2rgb(nr: c_int) -> RgbValue {
    /// The 216-colour cube's per-channel levels.
    const CUBE: [RgbValue; 6] = [0x00, 0x5f, 0x87, 0xaf, 0xd7, 0xff];
    /// The 24-step greyscale ramp above the cube.
    const GREY: [RgbValue; 24] = [
        0x08, 0x12, 0x1c, 0x26, 0x30, 0x3a, 0x44, 0x4e, 0x58, 0x62, 0x6c, 0x76, 0x80, 0x8a, 0x94,
        0x9e, 0xa8, 0xb2, 0xbc, 0xc6, 0xd0, 0xda, 0xe4, 0xee,
    ];
    /// The 16 ANSI colours: black, the six dark hues, light grey, then the
    /// bright half. (Upstream carries a fourth column of ANSI indices here,
    /// commented out at both its definition and its only reader.)
    const ANSI: [[RgbValue; 3]; 16] = [
        [0, 0, 0],
        [224, 0, 0],
        [0, 224, 0],
        [224, 224, 0],
        [0, 0, 224],
        [224, 0, 224],
        [0, 224, 224],
        [224, 224, 224],
        [128, 128, 128],
        [255, 64, 64],
        [64, 255, 64],
        [255, 255, 64],
        [64, 64, 255],
        [255, 64, 255],
        [64, 255, 255],
        [255, 255, 255],
    ];

    let [r, g, b] = if nr < 16 {
        ANSI[nr as usize]
    } else if nr < 232 {
        let idx = (nr - 16) as usize;
        [CUBE[idx / 36 % 6], CUBE[idx / 6 % 6], CUBE[idx % 6]]
    } else if nr < 256 {
        let level = GREY[(nr - 232) as usize];
        [level, level, level]
    } else {
        // Out of range: upstream leaves the channels at zero.
        [0, 0, 0]
    };
    (r << 16) + (g << 8) + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cterm2rgb_covers_the_three_ranges() {
        assert_eq!(cterm2rgb(1), 0xe00000); // ANSI dark red
        assert_eq!(cterm2rgb(16), 0x000000); // first cube entry
        assert_eq!(cterm2rgb(231), 0xffffff); // last cube entry
        assert_eq!(cterm2rgb(232), 0x080808); // first grey step
        assert_eq!(cterm2rgb(255), 0xeeeeee); // last grey step
        assert_eq!(cterm2rgb(256), 0); // past the end
    }

    #[test]
    fn the_ansi_and_ramp_tables_are_the_xterm_ones() {
        // A one-step change to either table is invisible through
        // `cterm_blend`, because `rgb2cterm` quantises the mixture back onto
        // the 216-colour cube -- so the tables are pinned here instead.
        let ansi: Vec<RgbValue> = (0..16).map(cterm2rgb).collect();
        assert_eq!(
            ansi,
            vec![
                0x000000, 0xe00000, 0x00e000, 0xe0e000, 0x0000e0, 0xe000e0, 0x00e0e0, 0xe0e0e0,
                0x808080, 0xff4040, 0x40ff40, 0xffff40, 0x4040ff, 0xff40ff, 0x40ffff, 0xffffff,
            ]
        );
        let ramp: Vec<RgbValue> = (232..256).map(cterm2rgb).collect();
        let steps: Vec<RgbValue> = (0..24)
            .map(|i| 0x08 + i * 0x0a)
            .map(|c| (c << 16) + (c << 8) + c)
            .collect();
        assert_eq!(ramp, steps);
        // The cube's six levels, read off the red channel of its first row.
        let cube: Vec<RgbValue> = (0..6).map(|i| cterm2rgb(16 + i * 36) >> 16).collect();
        assert_eq!(cube, vec![0x00, 0x5f, 0x87, 0xaf, 0xd7, 0xff]);
    }

    #[test]
    fn a_blend_ratio_picks_between_the_two_colours() {
        assert_eq!(rgb_blend(0, 0xff0000, 0x0000ff), 0x0000ff);
        assert_eq!(rgb_blend(100, 0xff0000, 0x0000ff), 0xff0000);
        assert_eq!(rgb_blend(50, 0x000000, 0xffffff), 0x7f7f7f);
    }

    #[test]
    fn rgb2cterm_answers_within_the_colour_cube() {
        assert_eq!(rgb2cterm(0x000000), 0);
        assert_eq!(rgb2cterm(0xffffff), 5 * 36 + 5 * 6 + 5);
    }
}
