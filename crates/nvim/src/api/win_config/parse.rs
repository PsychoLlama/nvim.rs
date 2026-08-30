//! Decoding the config keyset into a `WinConfig`.
//!
//! `parse_win_config` is the whole validation surface: which keys may appear
//! together, which are floats-only, which need a window or buffer handle, and
//! what each one's range is.  The small `parse_*` helpers are the individual
//! enumerated fields -- the anchor, the `relative` kind, the split direction,
//! the `bufpos` pair and the border title/footer with its position.
//!
//! The three pointers the family passes around get their names here as well
//! ([`CfgKeys`], [`WinCfg`], [`ErrSlot`]), and so do the safe spellings of the
//! validation messages ([`err_exp`] and friends).

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::validate::{self, Bad, err_expected, err_invalid, err_msg_ptr};
use crate::kvec::Kvec;
use crate::types::{ErrorType, NUL};
use crate::winfloat::WIN_CONFIG_INIT;
use crate::winlayer::{Live, Win};
use core::ffi::{CStr, c_char, c_int};
use core::mem::offset_of;
use core::ptr;

// ---------------------------------------------------------------------------
// The three pointers the family passes around
//
// Each is a [`Live<T>`](crate::winlayer::Live): a record that whoever built it
// promised the pointee outlives the value. Construction is the unsafe step,
// once per entry point; every field access after it is ordinary checked code,
// and the borrow `Deref` hands out lasts only as long as the access that
// asked for it.

/// The decoded `config` dictionary an entry point was handed.
pub(crate) type CfgKeys = Live<KeyDict_win_config>;

/// The window configuration being filled in from it.
pub(crate) type WinCfg = Live<WinConfig>;

/// The caller's error slot.
pub(crate) type ErrSlot = Live<Error>;

// ---------------------------------------------------------------------------
// The validation messages, written through the slot
//
// `api/private/validate.rs` answers with an `Error`; this family passes the
// caller's slot around as an [`ErrSlot`] instead of returning, so each of
// these is that answer stored. Every name they are handed is a literal and
// the slot is always the caller's own, so the promise is discharged once,
// here.

/// The slot as the `&mut Error` the shared helpers take.
///
/// [`ErrSlot`] is `Copy`, so `DerefMut` cannot hand one out without a `mut`
/// binding at every call site; this is that binding, made once. The lifetime
/// is unbounded, which is `cstr`'s convention for the same reason: `Live`'s
/// constructor already promised the slot outlives every use of the value.
pub(crate) fn slot_mut<'a>(err: ErrSlot) -> &'a mut Error {
    // SAFETY: `Live`'s promise, spent here.
    unsafe { &mut *err.raw() }
}

/// Store `e` in the caller's slot -- what every reporter here ends with, and
/// what a call site with a message of its own spells directly.
pub(crate) fn store(err: ErrSlot, e: Error) {
    // SAFETY: `err` names a live slot, which is what `Live` records.
    unsafe { *err.raw() = e };
}

/// "Invalid `name`: '`val`'", naming the keyset string that was wrong.
///
/// # Safety
/// `val`'s bytes must be NUL-terminated.
pub(crate) unsafe fn err_invalid_str(err: ErrSlot, name: &CStr, val: String_0, quote_val: bool) {
    // SAFETY: the caller's promise about `val`.
    let val = unsafe { crate::cstr::at_opt(val.data()) };
    let bad = match (val, quote_val) {
        (None, _) => Bad::Number(0),
        (Some(val), true) => Bad::Quoted(val),
        (Some(val), false) => Bad::Bare(val),
    };
    store(err, err_invalid(name, bad));
}

/// "Invalid `name`: expected `expected`", naming what arrived when `actual`
/// says.
pub(crate) fn err_exp(err: ErrSlot, name: &CStr, expected: &CStr, actual: Option<&CStr>) {
    store(err, err_expected(name, expected, actual));
}

/// "Required: `name`", for a key the caller left out.
pub(crate) fn err_required(err: ErrSlot, name: &CStr) {
    store(err, validate::err_required(name));
}

/// "Conflict: `name` not allowed with `name2`", for two keys that exclude
/// each other.
pub(crate) fn err_conflict(err: ErrSlot, name: &CStr, name2: &CStr) {
    store(err, validate::err_conflict(name, name2));
}

/// A failure of kind `kind` whose whole message is `msg`.
pub(crate) fn err_msg(err: ErrSlot, kind: ErrorType, msg: &CStr) {
    store(err, Error::from_message(kind, msg));
}

/// [`err_msg`] for the messages `main`'s statics hold rather than a literal.
///
/// # Safety
/// `msg` must be NUL-terminated.
pub(crate) unsafe fn err_msg_raw(err: ErrSlot, kind: ErrorType, msg: *const c_char) {
    // SAFETY: the caller's promise.
    store(err, unsafe { err_msg_ptr(kind, msg) });
}

// ---------------------------------------------------------------------------
// The enumerated keys

/// The index of the first of `names` that `s` spells, ignoring case.
///
/// # Safety
/// `s`'s bytes must be NUL-terminated.
unsafe fn imatch(s: String_0, names: &[&CStr]) -> Option<usize> {
    names
        .iter()
        // SAFETY: the caller's promise about `s`.
        .position(|name| unsafe { striequal(s.data(), name.as_ptr()) })
}

/// The `anchor` key: which corner of the float `row`/`col` place.
///
/// # Safety
/// `anchor`'s bytes must be NUL-terminated.
unsafe fn parse_float_anchor(anchor: String_0, out: &mut FloatAnchor) -> bool {
    if anchor.is_empty() {
        // NW is the default, and is neither bit.
        *out = 0;
    }
    // SAFETY: the caller's promise.
    let Some(which) = (unsafe { imatch(anchor, &[c"NW", c"NE", c"SW", c"SE"]) }) else {
        return false;
    };
    *out = [
        0,
        kFloatAnchorEast,
        kFloatAnchorSouth,
        kFloatAnchorSouth | kFloatAnchorEast,
    ][which];
    true
}

/// The `relative` key: what `row`/`col` are measured from.
///
/// # Safety
/// `relative`'s bytes must be NUL-terminated.
unsafe fn parse_float_relative(relative: String_0, out: &mut FloatRelative) -> bool {
    const NAMES: [&CStr; 6] = [
        c"editor",
        c"win",
        c"cursor",
        c"mouse",
        c"tabline",
        c"laststatus",
    ];
    // SAFETY: the caller's promise.
    let Some(which) = (unsafe { imatch(relative, &NAMES) }) else {
        return false;
    };
    *out = [
        kFloatRelativeEditor,
        kFloatRelativeWindow,
        kFloatRelativeCursor,
        kFloatRelativeMouse,
        kFloatRelativeTabline,
        kFloatRelativeLaststatus,
    ][which];
    true
}

/// The `split` key: which side of the target window the new one goes.
///
/// # Safety
/// `split`'s bytes must be NUL-terminated.
unsafe fn parse_config_split(split: String_0, out: &mut WinSplit) -> bool {
    const NAMES: [&CStr; 4] = [c"left", c"right", c"above", c"below"];
    // SAFETY: the caller's promise.
    let Some(which) = (unsafe { imatch(split, &NAMES) }) else {
        return false;
    };
    *out = [
        kWinSplitLeft,
        kWinSplitRight,
        kWinSplitAbove,
        kWinSplitBelow,
    ][which];
    true
}

/// The `bufpos` key: the `[lnum, col]` pair a `relative='win'` float hangs
/// off.
///
/// # Safety
/// `bufpos` must name its own `size` items.
unsafe fn parse_float_bufpos(bufpos: Array, out: &mut lpos_T) -> bool {
    if bufpos.size != 2 {
        return false;
    }
    // SAFETY: the caller's promise -- the array holds the two items read here.
    let (lnum, col) = unsafe { (*bufpos.items, *bufpos.items.add(1)) };
    if lnum.type_0 != kObjectTypeInteger || col.type_0 != kObjectTypeInteger {
        return false;
    }
    // SAFETY: the tags above say the integer arm of each is the live one.
    unsafe {
        out.lnum = lnum.data.integer as linenr_T;
        out.col = col.data.integer as colnr_T;
    }
    true
}

// ---------------------------------------------------------------------------
// The border text

/// The three `WinConfig` fields one of the two border texts is spelled in:
/// whether it is present, its chunks and its display width.
///
/// The addresses come off the config's raw pointer rather than off one
/// `Deref`, which is what lets all three stay usable at once -- see
/// [`Live`]'s module docs.
fn bordertext_fields(
    fconfig: WinCfg,
    which: BorderTextType,
) -> (Live<bool>, Live<VirtText>, Live<c_int>) {
    let (present, chunks, width) = if which == kBorderTextFooter {
        (
            offset_of!(WinConfig, footer),
            offset_of!(WinConfig, footer_chunks),
            offset_of!(WinConfig, footer_width),
        )
    } else {
        (
            offset_of!(WinConfig, title),
            offset_of!(WinConfig, title_chunks),
            offset_of!(WinConfig, title_width),
        )
    };
    // SAFETY: the three offsets name fields of the config `fconfig`'s builder
    // promised is live, so each address is live exactly as long as it is.
    unsafe {
        (
            Live::new(fconfig.field_ptr(present)),
            Live::new(fconfig.field_ptr(chunks)),
            Live::new(fconfig.field_ptr(width)),
        )
    }
}

/// The `title`/`footer` key: either one plain string or [`parse_virt_text`]'s
/// chunks.
///
/// # Safety
/// A `String` `bordertext` must be NUL-terminated, and an `Array` one must
/// name its own items.
unsafe fn parse_bordertext(
    bordertext: Object,
    bordertext_type: BorderTextType,
    fconfig: WinCfg,
    err: ErrSlot,
) {
    if bordertext.type_0 != kObjectTypeString && bordertext.type_0 != kObjectTypeArray {
        let actual = api_typename(bordertext.type_0);
        err_exp(err, c"title/footer", c"String or Array", Some(actual));
        return;
    }
    let chunk_array = if bordertext.type_0 == kObjectTypeArray {
        // SAFETY: the tag above says the array arm is the live one.
        let array = unsafe { bordertext.data.array };
        if array.size == 0 {
            err_exp(err, c"title/footer", c"non-empty Array", None);
            return;
        }
        Some(array)
    } else {
        None
    };
    let (mut is_present, mut chunks, mut width) = bordertext_fields(fconfig, bordertext_type);
    let Some(array) = chunk_array else {
        // SAFETY: the tag says the string arm is the live one.
        let string = unsafe { bordertext.data.string };
        if string.is_empty() {
            *is_present = false;
            return;
        }
        // `kv_init` and then the `kv_push` whose growth step c2rust expanded
        // inline, on this frame's own vector rather than in place: three
        // `&mut`s into the config at once is what `Live` cannot give.
        let mut text = VirtText {
            size: 0,
            capacity: 0,
            items: ptr::null_mut::<VirtTextChunk>(),
        };
        // SAFETY: the caller's promise -- `string` is NUL-terminated -- and
        // `text` is this frame's own empty vector.
        unsafe {
            let hl_id = -1;
            let chunk = VirtTextChunk {
                text: xstrdup(string.data()),
                hl_id,
            };
            Kvec::new(&mut text.size, &mut text.capacity, &mut text.items).push(chunk);
        }
        // SAFETY: as above.
        *width = unsafe { mb_string2cells(string.data()) } as c_int;
        *chunks = text;
        *is_present = true;
        return;
    };
    *width = 0;
    // SAFETY: the caller's promise about the array, and both out-parameters
    // name fields of the config.
    *chunks = unsafe { parse_virt_text(array, slot_mut(err), width.raw()) };
    *is_present = true;
}

/// The `title_pos`/`footer_pos` key: which end of the border the text sits
/// at.
///
/// # Safety
/// `bordertext_pos`'s bytes must be NUL-terminated.
unsafe fn parse_bordertext_pos(
    wp: Option<Win>,
    bordertext_pos: String_0,
    bordertext_type: BorderTextType,
    fconfig: WinCfg,
    err: ErrSlot,
) -> bool {
    let align = if bordertext_type == kBorderTextFooter {
        offset_of!(WinConfig, footer_pos)
    } else {
        offset_of!(WinConfig, title_pos)
    };
    // SAFETY: the offset names a field of the config `fconfig`'s builder
    // promised is live.
    let mut align: Live<AlignTextPos> = unsafe { Live::new(fconfig.field_ptr(align)) };
    if bordertext_pos.is_empty() {
        // A new window starts left-aligned; an existing one keeps what it
        // had.
        if wp.is_none() {
            *align = kAlignLeft;
        }
        return true;
    }
    const NAMES: [&CStr; 3] = [c"left", c"center", c"right"];
    // SAFETY: the caller's promise.
    let Some(which) = (unsafe { smatch(bordertext_pos, &NAMES) }) else {
        let name = if bordertext_type == kBorderTextTitle {
            c"title_pos"
        } else {
            c"footer_pos"
        };
        // SAFETY: as above.
        unsafe { err_invalid_str(err, name, bordertext_pos, true) };
        return false;
    };
    *align = [kAlignLeft, kAlignCenter, kAlignRight][which];
    true
}

/// [`imatch`], case-sensitively.
///
/// # Safety
/// `s`'s bytes must be NUL-terminated.
unsafe fn smatch(s: String_0, names: &[&CStr]) -> Option<usize> {
    names
        .iter()
        // SAFETY: the caller's promise about `s`.
        .position(|name| unsafe { strequal(s.data(), name.as_ptr()) })
}

// ---------------------------------------------------------------------------
// The whole keyset

/// The current window, which exists from startup to exit.
fn cur_win() -> Win {
    // SAFETY: `curwin` names a live window for the editor's whole run.
    unsafe { Win::current() }
}

/// Fill `fconfig` in from `config`, reporting the first thing wrong with it
/// through `err`.
///
/// `wp` is the window being reconfigured, `None` when one is being created;
/// `reconf` says that the missing keys keep whatever the window already had
/// rather than being required.
///
/// On failure `fconfig` is merged back onto the window's current config (or
/// onto the defaults) before `false` is answered, so a rejected call leaves a
/// usable config behind.
///
/// # Safety
/// Every string in `config` must be NUL-terminated and every array must name
/// its own items, which is what the keyset decoder guarantees.
pub(crate) unsafe fn parse_win_config(
    wp: Option<Win>,
    config: CfgKeys,
    mut fconfig: WinCfg,
    reconf: bool,
    err: ErrSlot,
) -> bool {
    let keys = config.is_set__win_config_;
    let set = |key| has_key(keys, key);
    let floating = |w: &Win| w.w_floating;
    let mut has_relative = false;
    let mut relative_is_win = false;
    let mut is_split = false;
    '_fail: {
        if !config.relative.is_empty() {
            // SAFETY: the caller's promise -- the keyset's strings are
            // NUL-terminated.
            if !unsafe { parse_float_relative(config.relative, &mut fconfig.relative) } {
                // SAFETY: as above.
                unsafe { err_invalid_str(err, c"relative", config.relative, true) };
                break '_fail;
            }
            if !(set(KEYSET_OPTIDX_win_config__row) && set(KEYSET_OPTIDX_win_config__col))
                && !set(KEYSET_OPTIDX_win_config__bufpos)
            {
                err_required(err, c"'relative' requires 'row'/'col' or 'bufpos'");
                break '_fail;
            }
            has_relative = true;
            fconfig.external = false;
            if fconfig.relative == kFloatRelativeWindow {
                relative_is_win = true;
                fconfig.bufpos.lnum = -1;
            }
        } else if !config.external {
            if set(KEYSET_OPTIDX_win_config__vertical) || set(KEYSET_OPTIDX_win_config__split) {
                is_split = true;
                fconfig.external = false;
            } else if wp.is_none() {
                err_required(err, c"'relative' or 'external' when creating a float");
                break '_fail;
            }
        }
        // A split-only key on a float, and a float-only key on a split, are
        // both reported here rather than in the walk below.
        if set(KEYSET_OPTIDX_win_config__vertical) && !is_split {
            err_conflict(err, c"vertical", c"floating windows");
            break '_fail;
        }
        if set(KEYSET_OPTIDX_win_config__split) && !is_split {
            err_conflict(err, c"split", c"floating windows");
            break '_fail;
        }
        if set(KEYSET_OPTIDX_win_config__split) {
            // SAFETY: the caller's promise about the keyset's strings.
            if !unsafe { parse_config_split(config.split, &mut fconfig.split) } {
                // SAFETY: as above.
                unsafe { err_invalid_str(err, c"split", config.split, true) };
                break '_fail;
            }
        }
        if set(KEYSET_OPTIDX_win_config__anchor) {
            // SAFETY: as above.
            if !unsafe { parse_float_anchor(config.anchor, &mut fconfig.anchor) } {
                // SAFETY: as above.
                unsafe { err_invalid_str(err, c"anchor", config.anchor, true) };
                break '_fail;
            }
        }
        if set(KEYSET_OPTIDX_win_config__row) {
            if !has_relative || is_split {
                generate_error(wp, c"row", err);
                break '_fail;
            }
            fconfig.row = config.row;
        }
        if set(KEYSET_OPTIDX_win_config__col) {
            if !has_relative || is_split {
                generate_error(wp, c"col", err);
                break '_fail;
            }
            fconfig.col = config.col;
        }
        if set(KEYSET_OPTIDX_win_config__bufpos) {
            if !has_relative || is_split {
                generate_error(wp, c"bufpos", err);
                break '_fail;
            }
            // SAFETY: the caller's promise -- the keyset's arrays name their
            // own items.
            if !unsafe { parse_float_bufpos(config.bufpos, &mut fconfig.bufpos) } {
                err_exp(err, c"bufpos", c"[row, col] array", None);
                break '_fail;
            }
            // `bufpos` without `row`/`col` puts the float just below the
            // position, or just above it for a south anchor.
            if !set(KEYSET_OPTIDX_win_config__row) {
                fconfig.row = if fconfig.anchor & kFloatAnchorSouth != 0 {
                    0.0
                } else {
                    1.0
                };
            }
            if !set(KEYSET_OPTIDX_win_config__col) {
                fconfig.col = 0.0;
            }
        }
        if set(KEYSET_OPTIDX_win_config__width) {
            if config.width <= 0 {
                err_exp(err, c"width", c"positive Integer", None);
                break '_fail;
            }
            fconfig.width = config.width as c_int;
        } else if !reconf && !is_split {
            err_required(err, c"width");
            break '_fail;
        }
        if set(KEYSET_OPTIDX_win_config__height) {
            if config.height <= 0 {
                err_exp(err, c"height", c"positive Integer", None);
                break '_fail;
            }
            fconfig.height = config.height as c_int;
        } else if !reconf && !is_split {
            err_required(err, c"height");
            break '_fail;
        }
        if set(KEYSET_OPTIDX_win_config__external) {
            fconfig.external = config.external;
            if has_relative && fconfig.external {
                err_conflict(err, c"relative", c"external");
                break '_fail;
            }
            if fconfig.external && !ui_has(kUIMultigrid) {
                let msg = c"UI doesn't support external windows";
                err_msg(err, kErrorTypeValidation, msg);
                break '_fail;
            }
        }
        if set(KEYSET_OPTIDX_win_config__win) && fconfig.external {
            err_conflict(err, c"win", c"external window");
            break '_fail;
        }
        let win_is_target = set(KEYSET_OPTIDX_win_config__win)
            && !is_split
            && wp.as_ref().is_some_and(floating)
            && fconfig.relative == kFloatRelativeWindow;
        if relative_is_win || win_is_target {
            // SAFETY: `err` names a live error slot, and the lookup answers a
            // live window or null.
            let target = unsafe { Win::from_raw(find_window_by_handle(config.win, slot_mut(err))) };
            let Some(target) = target else {
                break '_fail;
            };
            if Some(target) == wp {
                let msg = c"floating window cannot be relative to itself";
                err_msg(err, kErrorTypeException, msg);
                break '_fail;
            }
            fconfig.window = target.handle;
        } else {
            if set(KEYSET_OPTIDX_win_config__win) {
                if !is_split && !has_relative && !wp.as_ref().is_some_and(floating) {
                    err_required(err, c"non-float with 'win' requires 'split' or 'vertical'");
                    break '_fail;
                }
                fconfig.window = config.win;
            }
            if fconfig.window == 0 {
                fconfig.window = cur_win().handle;
            }
        }
        if set(KEYSET_OPTIDX_win_config__focusable) {
            fconfig.focusable = config.focusable;
            fconfig.mouse = config.focusable;
        }
        if set(KEYSET_OPTIDX_win_config__mouse) {
            fconfig.mouse = config.mouse;
        }
        if set(KEYSET_OPTIDX_win_config__zindex) {
            if is_split {
                err_conflict(err, c"zindex", c"non-float window");
                break '_fail;
            }
            if config.zindex <= 0 {
                err_exp(err, c"zindex", c"positive Integer", None);
                break '_fail;
            }
            fconfig.zindex = config.zindex as c_int;
        }
        if set(KEYSET_OPTIDX_win_config__title) {
            if is_split {
                err_conflict(err, c"title", c"non-float window");
                break '_fail;
            }
            // SAFETY: the caller's promise about the keyset's strings and
            // arrays.
            let placed = unsafe {
                parse_bordertext(config.title, kBorderTextTitle, fconfig, err);
                !err.is_set()
                    && parse_bordertext_pos(wp, config.title_pos, kBorderTextTitle, fconfig, err)
            };
            if !placed {
                break '_fail;
            }
        } else if set(KEYSET_OPTIDX_win_config__title_pos) {
            err_required(err, c"'title' requires 'title_pos'");
            break '_fail;
        }
        if set(KEYSET_OPTIDX_win_config__footer) {
            if is_split {
                err_conflict(err, c"footer", c"non-float window");
                break '_fail;
            }
            // SAFETY: as the title above.
            let placed = unsafe {
                parse_bordertext(config.footer, kBorderTextFooter, fconfig, err);
                !err.is_set()
                    && parse_bordertext_pos(wp, config.footer_pos, kBorderTextFooter, fconfig, err)
            };
            if !placed {
                break '_fail;
            }
        } else if set(KEYSET_OPTIDX_win_config__footer_pos) {
            err_required(err, c"'footer' requires 'footer_pos'");
            break '_fail;
        }
        if set(KEYSET_OPTIDX_win_config__border) {
            if is_split {
                err_conflict(err, c"border", c"non-float window");
                break '_fail;
            }
            let border_style = config.border;
            if border_style.type_0 != kObjectTypeNil {
                // SAFETY: the caller's promise about the keyset's strings and
                // arrays, and `fconfig` and `err` are live.
                unsafe { parse_border_style(border_style, fconfig.raw(), slot_mut(err)) };
                if err.is_set() {
                    break '_fail;
                }
            }
        } else if !wp.as_ref().is_some_and(floating) {
            // No `border` key on a new float: `'winborder'` decides.
            // SAFETY: the option's value is a live NUL-terminated string, and
            // `fconfig` and `err` are live.
            let winborder = unsafe { *p_winborder.get() };
            if winborder as c_int != NUL
                // SAFETY: as above.
                && !unsafe { parse_winborder(fconfig.raw(), p_winborder.get(), slot_mut(err)) }
            {
                break '_fail;
            }
        }
        if set(KEYSET_OPTIDX_win_config__style) {
            // SAFETY: the caller's promise -- the keyset's strings are
            // NUL-terminated.
            let empty = unsafe { *config.style.data() } as c_int == NUL;
            // SAFETY: as above.
            let minimal = !empty && unsafe { imatch(config.style, &[c"minimal"]) }.is_some();
            if empty {
                fconfig.style = kWinStyleUnused;
            } else if minimal {
                fconfig.style = kWinStyleMinimal;
            } else {
                // SAFETY: as above.
                unsafe { err_invalid_str(err, c"style", config.style, true) };
                break '_fail;
            }
        }
        if set(KEYSET_OPTIDX_win_config__noautocmd) {
            if wp.is_some() && config.noautocmd != fconfig.noautocmd {
                let msg = c"'noautocmd' cannot be changed on existing window";
                err_msg(err, kErrorTypeValidation, msg);
                break '_fail;
            }
            fconfig.noautocmd = config.noautocmd;
        }
        if set(KEYSET_OPTIDX_win_config__fixed) {
            fconfig.fixed = config.fixed;
        }
        if set(KEYSET_OPTIDX_win_config__hide) {
            fconfig.hide = config.hide;
        }
        if set(KEYSET_OPTIDX_win_config___cmdline_offset) {
            fconfig._cmdline_offset = config._cmdline_offset as c_int;
        }
        return true;
    }
    let base = wp.map_or(WIN_CONFIG_INIT, |w| w.w_config.clone());
    // SAFETY: `fconfig` names the live config the caller promised.
    unsafe { merge_win_config(fconfig.raw(), base) };
    false
}

/// [`generate_api_error`] with the window as a handle and the name as a
/// literal: "this key needs a `relative`", or "not on a split".
fn generate_error(wp: Option<Win>, attribute: &CStr, err: ErrSlot) {
    let wp = wp.map_or(ptr::null_mut(), Win::raw);
    // SAFETY: `wp` is null or a live window, and `err` names a live slot.
    unsafe { generate_api_error(wp, attribute.as_ptr(), slot_mut(err)) };
}
