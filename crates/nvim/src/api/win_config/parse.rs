//! Decoding the config keyset into a `WinConfig`.
//!
//! [`parse_win_config`] is the whole validation surface: which keys may appear
//! together, which are floats-only, which need a window or buffer handle, and
//! what each one's range is.  It is one [`Decode`] per call, and one method
//! per group of keys -- where the window hangs, where it sits, how big it is,
//! which window it belongs to, its border and its border text -- each
//! answering a [`Result`] whose `Err` is the first thing found wrong.
//!
//! The two pointers the family passes around get their names here as well
//! ([`CfgKeys`] and [`WinCfg`]).

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::validate::{Bad, err_conflict, err_expected, err_invalid, err_required};
use crate::kvec::Kvec;
use crate::winfloat::WIN_CONFIG_INIT;
use crate::winlayer::{Live, Win};
use core::ffi::{CStr, c_int};
use core::ptr;

// ---------------------------------------------------------------------------
// The two pointers the family passes around
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

/// What a parse that found nothing wrong with the keyset came to.
///
/// Not a two-valued answer, because "nothing was wrong" and "here is a config
/// to act on" are not the same thing. Two paths refuse without a word to say
/// about it -- a `win` handle that resolves to no window at all, and a
/// `'winborder'` value that does not spell a border -- and upstream answers
/// both by making no window and reporting nothing. `Err` is the other half:
/// a message for the client, which is every other refusal here.
pub(crate) enum Parsed {
    /// The keyset was decoded into the config.
    Done,
    /// Nothing was decoded and there is nothing to report.
    Refused,
}

// ---------------------------------------------------------------------------
// The validation messages
//
// `api/private/validate.rs` answers with an `Error` and so does every helper
// here; the only one that needs a spelling of its own is the one that quotes
// a keyset string back at the client.

/// "Invalid `name`: '`value`'", naming the keyset string that was wrong.
///
/// The null string has no bytes to quote, and upstream's `%s` over a null
/// pointer is why it reads back as a number instead.
fn err_invalid_str(name: &CStr, value: &String_0) -> Error {
    let bad = if value.is_null() {
        Bad::Number(0)
    } else {
        Bad::Quoted(value.as_cstr())
    };
    err_invalid(name, bad)
}

// ---------------------------------------------------------------------------
// The enumerated keys
//
// One function per key that spells its value as a name, each answering the
// value that name stands for. A `None` is the caller's message to write:
// which key was wrong is not something these know.

/// The index of the first of `names` that `spelling` spells, ignoring case.
fn imatch(spelling: &CStr, names: &[&CStr]) -> Option<usize> {
    // SAFETY: both sides are `CStr`s, which is the NUL-terminated string
    // `striequal` reads. It is the locale's case fold, not ASCII's, which is
    // why this is not `eq_ignore_ascii_case`.
    names
        .iter()
        .position(|name| unsafe { striequal(spelling.as_ptr(), name.as_ptr()) })
}

/// The `anchor` key: which corner of the float `row`/`col` place.
fn float_anchor(spelling: &CStr) -> Option<FloatAnchor> {
    const NAMES: [&CStr; 4] = [c"NW", c"NE", c"SW", c"SE"];
    // NW is the default, and is neither bit.
    const CORNERS: [FloatAnchor; 4] = [
        0,
        kFloatAnchorEast,
        kFloatAnchorSouth,
        kFloatAnchorSouth | kFloatAnchorEast,
    ];
    Some(CORNERS[imatch(spelling, &NAMES)?])
}

/// The `relative` key: what `row`/`col` are measured from.
fn float_relative(spelling: &CStr) -> Option<FloatRelative> {
    const NAMES: [&CStr; 6] = [
        c"editor",
        c"win",
        c"cursor",
        c"mouse",
        c"tabline",
        c"laststatus",
    ];
    const KINDS: [FloatRelative; 6] = [
        kFloatRelativeEditor,
        kFloatRelativeWindow,
        kFloatRelativeCursor,
        kFloatRelativeMouse,
        kFloatRelativeTabline,
        kFloatRelativeLaststatus,
    ];
    Some(KINDS[imatch(spelling, &NAMES)?])
}

/// The `split` key: which side of the target window the new one goes.
fn config_split(spelling: &CStr) -> Option<WinSplit> {
    const NAMES: [&CStr; 4] = [c"left", c"right", c"above", c"below"];
    const SIDES: [WinSplit; 4] = [
        kWinSplitLeft,
        kWinSplitRight,
        kWinSplitAbove,
        kWinSplitBelow,
    ];
    Some(SIDES[imatch(spelling, &NAMES)?])
}

/// The `title_pos`/`footer_pos` key: which end of the border the text sits
/// at. Case-sensitive, unlike every other name here.
fn align_pos(spelling: &CStr) -> Option<AlignTextPos> {
    const NAMES: [&CStr; 3] = [c"left", c"center", c"right"];
    const ENDS: [AlignTextPos; 3] = [kAlignLeft, kAlignCenter, kAlignRight];
    Some(ENDS[NAMES.iter().position(|name| *name == spelling)?])
}

/// The `bufpos` key: the `[lnum, col]` pair a `relative='win'` float hangs
/// off.
fn float_bufpos(bufpos: &Array) -> Option<LPos> {
    let [lnum, col] = bufpos.as_slice() else {
        return None;
    };
    Some(LPos {
        lnum: lnum.as_integer()? as LineNr,
        col: col.as_integer()? as ColNr,
    })
}

// ---------------------------------------------------------------------------
// The border text

/// The three `WinConfig` fields one of the two border texts is spelled in.
///
/// `chunks` and `width` are `None` where the key leaves whatever the window
/// already had: clearing the text with an empty string writes `present` and
/// nothing else, which is a distinction the three bare field writes this
/// replaces made only by leaving early.
struct BorderText {
    present: bool,
    chunks: Option<VirtText>,
    width: Option<c_int>,
}

impl BorderText {
    /// Write the three fields `which` names.
    ///
    /// The chunks the config already held are dropped on the floor rather
    /// than released, as upstream drops them: whatever reaches the window
    /// takes them over, and a refusal on the way there has
    /// [`merge_win_config`] free them.
    fn apply(self, config: &mut WinConfig, which: BorderTextType) {
        let (present, chunks, width) = if which == kBorderTextFooter {
            (
                &mut config.footer,
                &mut config.footer_chunks,
                &mut config.footer_width,
            )
        } else {
            (
                &mut config.title,
                &mut config.title_chunks,
                &mut config.title_width,
            )
        };
        *present = self.present;
        if let Some(text) = self.chunks {
            *chunks = text;
        }
        if let Some(cells) = self.width {
            *width = cells;
        }
    }
}

/// The `title`/`footer` key: either one plain string or [`parse_virt_text`]'s
/// chunks.
fn set_border_text(mut config: WinCfg, which: BorderTextType, text: &Object) -> Result<(), Error> {
    if text.as_array().is_some_and(|array| array.is_empty()) {
        return Err(err_expected(c"title/footer", c"non-empty Array", None));
    }
    let decoded = match text {
        // An empty string clears the text; its chunks and width are the
        // window's own until something replaces them.
        Object::String(string) if string.is_empty() => BorderText {
            present: false,
            chunks: None,
            width: None,
        },
        Object::String(string) => BorderText {
            present: true,
            chunks: Some(one_chunk(string)),
            // SAFETY: the string owns its NUL-terminated bytes.
            width: Some(unsafe { mb_string2cells(string.data()) } as c_int),
        },
        Object::Array(array) => {
            let mut cells = 0;
            let chunks = parse_virt_text(array, Some(&mut cells))?;
            BorderText {
                present: true,
                chunks: Some(chunks),
                width: Some(cells),
            }
        }
        other => {
            let actual = api_typename(other.kind());
            return Err(err_expected(
                c"title/footer",
                c"String or Array",
                Some(actual),
            ));
        }
    };
    decoded.apply(&mut config, which);
    Ok(())
}

/// One border text holding `string` and no highlight of its own.
///
/// `kv_init` and then the `kv_push` whose growth step c2rust expanded inline,
/// on this frame's own vector rather than in place: three `&mut`s into the
/// config at once is what [`Live`] cannot give.
fn one_chunk(string: &String_0) -> VirtText {
    let mut text = VirtText {
        size: 0,
        capacity: 0,
        items: ptr::null_mut::<VirtTextChunk>(),
    };
    // SAFETY: the string owns its NUL-terminated bytes, and `text` is this
    // frame's own empty vector.
    unsafe {
        let chunk = VirtTextChunk {
            text: xstrdup(string.data()),
            hl_id: -1,
        };
        Kvec::new(&mut text.size, &mut text.capacity, &mut text.items).push(chunk);
    }
    text
}

/// Write the `title_pos`/`footer_pos` field `which` names.
fn set_align(mut config: WinCfg, which: BorderTextType, align: AlignTextPos) {
    if which == kBorderTextFooter {
        config.footer_pos = align;
    } else {
        config.title_pos = align;
    }
}

// ---------------------------------------------------------------------------
// The whole keyset

/// Fill `fconfig` in from `config`, answering the first thing wrong with it.
///
/// `window` is the window being reconfigured, `None` when one is being
/// created; `reconf` says that the missing keys keep whatever the window
/// already had rather than being required.
///
/// Whatever the answer is not [`Parsed::Done`], `fconfig` is merged back onto
/// the window's current config (or onto the defaults) before it is given, so
/// a rejected call leaves a usable config behind.
pub(crate) fn parse_win_config(
    window: Option<Win>,
    config: CfgKeys,
    fconfig: WinCfg,
    reconf: bool,
) -> Result<Parsed, Error> {
    let mut decode = Decode {
        window,
        config,
        fconfig,
        reconf,
        has_relative: false,
        relative_is_win: false,
        is_split: false,
    };
    let answer = decode.run();
    if !matches!(answer, Ok(Parsed::Done)) {
        decode.restore();
    }
    answer
}

/// One `config` keyset being decoded into one `WinConfig`.
///
/// The three flags are what the `goto fail` this replaces shared between its
/// sections: each is settled by an earlier key and read by a later one, and
/// naming them together is what lets each section be a method of its own.
struct Decode {
    /// The window being reconfigured, `None` when one is being created.
    window: Option<Win>,
    /// The keyset the caller handed in.
    config: CfgKeys,
    /// The configuration being filled in from it.
    fconfig: WinCfg,
    /// Whether a key the caller left out keeps what the window had rather
    /// than being required.
    reconf: bool,
    /// Whether `relative` named something, and whether what it named is a
    /// window.
    has_relative: bool,
    relative_is_win: bool,
    /// Whether the config describes a split rather than a float.
    is_split: bool,
}

impl Decode {
    /// Every group of keys in the order upstream reads them, which is also
    /// the order their messages are reported in.
    fn run(&mut self) -> Result<Parsed, Error> {
        self.placement()?;
        self.position()?;
        self.size()?;
        if matches!(self.target_window()?, Parsed::Refused) {
            return Ok(Parsed::Refused);
        }
        self.focus();
        self.zindex()?;
        self.border_text()?;
        if matches!(self.border()?, Parsed::Refused) {
            return Ok(Parsed::Refused);
        }
        self.style()?;
        self.flags()?;
        Ok(Parsed::Done)
    }

    /// Merge the window's own config -- or the defaults, for a window that
    /// does not exist yet -- back over whatever was filled in.
    fn restore(&self) {
        let base = self.window.map_or(WIN_CONFIG_INIT, |w| w.w_config.clone());
        // SAFETY: `self.fconfig` names the live config its builder promised.
        unsafe { merge_win_config(self.fconfig.raw(), base) };
    }

    /// Whether `window` is a float, which several keys read differently.
    fn on_a_float(&self) -> bool {
        self.window.is_some_and(|w| w.w_floating)
    }

    /// The `relative`, `external`, `split`, `vertical` and `anchor` keys:
    /// what the window hangs off, and which way round.
    fn placement(&mut self) -> Result<(), Error> {
        match self.config.relative.as_ref().filter(|r| !r.is_empty()) {
            Some(relative) => {
                let Some(kind) = float_relative(relative.as_cstr()) else {
                    return Err(err_invalid_str(c"relative", relative));
                };
                self.fconfig.relative = kind;
                if !(self.config.row.is_some() && self.config.col.is_some())
                    && self.config.bufpos.is_none()
                {
                    return Err(err_required(c"'relative' requires 'row'/'col' or 'bufpos'"));
                }
                self.has_relative = true;
                self.fconfig.external = false;
                if kind == kFloatRelativeWindow {
                    self.relative_is_win = true;
                    self.fconfig.bufpos.lnum = -1;
                }
            }
            None if !self.config.external.unwrap_or(false) => {
                if self.config.vertical.is_some() || self.config.split.is_some() {
                    self.is_split = true;
                    self.fconfig.external = false;
                } else if self.window.is_none() {
                    return Err(err_required(
                        c"'relative' or 'external' when creating a float",
                    ));
                }
            }
            None => {}
        }
        // A split-only key on a float, and a float-only key on a split, are
        // both reported here rather than where the key itself is read.
        if self.config.vertical.is_some() && !self.is_split {
            return Err(err_conflict(c"vertical", c"floating windows"));
        }
        if self.config.split.is_some() && !self.is_split {
            return Err(err_conflict(c"split", c"floating windows"));
        }
        if let Some(split) = self.config.split.as_ref() {
            let Some(side) = config_split(split.as_cstr()) else {
                return Err(err_invalid_str(c"split", split));
            };
            self.fconfig.split = side;
        }
        if let Some(anchor) = self.config.anchor.as_ref() {
            let Some(corner) = float_anchor(anchor.as_cstr()) else {
                return Err(err_invalid_str(c"anchor", anchor));
            };
            self.fconfig.anchor = corner;
        }
        Ok(())
    }

    /// The `row`, `col` and `bufpos` keys: where a float sits.
    fn position(&mut self) -> Result<(), Error> {
        // All three are float-only, and all three say so the same way: a
        // window being reconfigured is told it needs a `relative`, and a
        // split that the key is not one a split has.
        let placed = self.has_relative && !self.is_split;
        if let Some(row) = self.config.row {
            if !placed {
                return Err(generate_api_error(self.window, c"row"));
            }
            self.fconfig.row = row;
        }
        if let Some(col) = self.config.col {
            if !placed {
                return Err(generate_api_error(self.window, c"col"));
            }
            self.fconfig.col = col;
        }
        let Some(bufpos) = self.config.bufpos.as_ref() else {
            return Ok(());
        };
        if !placed {
            return Err(generate_api_error(self.window, c"bufpos"));
        }
        let Some(at) = float_bufpos(bufpos) else {
            return Err(err_expected(c"bufpos", c"[row, col] array", None));
        };
        self.fconfig.bufpos = at;
        // `bufpos` without `row`/`col` puts the float just below the
        // position, or just above it for a south anchor.
        if self.config.row.is_none() {
            self.fconfig.row = if self.fconfig.anchor & kFloatAnchorSouth != 0 {
                0.0
            } else {
                1.0
            };
        }
        if self.config.col.is_none() {
            self.fconfig.col = 0.0;
        }
        Ok(())
    }

    /// The `width` and `height` keys, which a new float must carry and a
    /// split takes from the layout.
    fn size(&mut self) -> Result<(), Error> {
        self.fconfig.width = self.one_size(self.config.width, c"width", self.fconfig.width)?;
        self.fconfig.height = self.one_size(self.config.height, c"height", self.fconfig.height)?;
        Ok(())
    }

    /// One of the two: `given` when it is a positive integer, `had` when the
    /// key may be left out at all.
    fn one_size(&self, given: Option<Integer>, name: &CStr, had: c_int) -> Result<c_int, Error> {
        match given {
            Some(size) if size <= 0 => Err(err_expected(name, c"positive Integer", None)),
            Some(size) => Ok(size as c_int),
            None if !self.reconf && !self.is_split => Err(err_required(name)),
            None => Ok(had),
        }
    }

    /// The `external` and `win` keys: whether the window is the UI's rather
    /// than the editor's, and which window it hangs off if it is not.
    fn target_window(&mut self) -> Result<Parsed, Error> {
        if let Some(external) = self.config.external {
            self.fconfig.external = external;
            if self.has_relative && external {
                return Err(err_conflict(c"relative", c"external"));
            }
            if external && !ui_has(kUIMultigrid) {
                return Err(Error::validation(c"UI doesn't support external windows"));
            }
        }
        if self.config.win.is_some() && self.fconfig.external {
            return Err(err_conflict(c"win", c"external window"));
        }
        let win_is_target = self.config.win.is_some()
            && !self.is_split
            && self.on_a_float()
            && self.fconfig.relative == kFloatRelativeWindow;
        if !(self.relative_is_win || win_is_target) {
            return self.split_target();
        }
        let Some(target) = find_window_by_handle(self.config.win.unwrap_or(0))? else {
            return Ok(Parsed::Refused);
        };
        if Some(target) == self.window {
            let why = c"floating window cannot be relative to itself";
            return Err(Error::exception(why));
        }
        self.fconfig.window = target.handle;
        Ok(Parsed::Done)
    }

    /// The `win` key where it names the window to split rather than the one
    /// to float over. Left where it is, since a handle is not resolved here.
    fn split_target(&mut self) -> Result<Parsed, Error> {
        if let Some(win_handle) = self.config.win {
            if !self.is_split && !self.has_relative && !self.on_a_float() {
                return Err(err_required(
                    c"non-float with 'win' requires 'split' or 'vertical'",
                ));
            }
            self.fconfig.window = win_handle;
        }
        if self.fconfig.window == 0 {
            self.fconfig.window = Win::current().handle;
        }
        Ok(Parsed::Done)
    }

    /// The `focusable` and `mouse` keys. `focusable` carries `mouse` with
    /// it, and a `mouse` of its own overrides that.
    fn focus(&mut self) {
        if let Some(focusable) = self.config.focusable {
            self.fconfig.focusable = focusable;
            self.fconfig.mouse = focusable;
        }
        if let Some(mouse) = self.config.mouse {
            self.fconfig.mouse = mouse;
        }
    }

    /// The `zindex` key: which floats draw over which.
    fn zindex(&mut self) -> Result<(), Error> {
        let Some(zindex) = self.config.zindex else {
            return Ok(());
        };
        if self.is_split {
            return Err(err_conflict(c"zindex", c"non-float window"));
        }
        if zindex <= 0 {
            return Err(err_expected(c"zindex", c"positive Integer", None));
        }
        self.fconfig.zindex = zindex as c_int;
        Ok(())
    }

    /// The `title`/`footer` keys and the `*_pos` each of them takes.
    fn border_text(&mut self) -> Result<(), Error> {
        self.one_border_text(kBorderTextTitle)?;
        self.one_border_text(kBorderTextFooter)
    }

    /// One of the two, whichever `which` names.
    fn one_border_text(&mut self, which: BorderTextType) -> Result<(), Error> {
        let footer = which == kBorderTextFooter;
        let (name, text, pos) = if footer {
            (c"footer", &self.config.footer, &self.config.footer_pos)
        } else {
            (c"title", &self.config.title, &self.config.title_pos)
        };
        let Some(text) = text.as_ref() else {
            // The position on its own places nothing.
            if pos.is_none() {
                return Ok(());
            }
            let why = if footer {
                c"'footer' requires 'footer_pos'"
            } else {
                c"'title' requires 'title_pos'"
            };
            return Err(err_required(why));
        };
        if self.is_split {
            return Err(err_conflict(name, c"non-float window"));
        }
        set_border_text(self.fconfig, which, text)?;
        let Some(pos) = pos.as_ref().filter(|p| !p.is_empty()) else {
            // A new window starts left-aligned; an existing one keeps what
            // it had.
            if self.window.is_none() {
                set_align(self.fconfig, which, kAlignLeft);
            }
            return Ok(());
        };
        let Some(end) = align_pos(pos.as_cstr()) else {
            let name = if footer { c"footer_pos" } else { c"title_pos" };
            return Err(err_invalid_str(name, pos));
        };
        set_align(self.fconfig, which, end);
        Ok(())
    }

    /// The `border` key, or `'winborder'` where a new float leaves it out.
    fn border(&mut self) -> Result<Parsed, Error> {
        let Some(style) = self.config.border.as_ref() else {
            return self.winborder();
        };
        if self.is_split {
            return Err(err_conflict(c"border", c"non-float window"));
        }
        if !style.is_nil() {
            // SAFETY: `self.fconfig` names the live config its builder
            // promised.
            unsafe { parse_border_style(style, self.fconfig.raw()) }?;
        }
        Ok(Parsed::Done)
    }

    /// No `border` key on a new float: `'winborder'` decides, and a value
    /// that does not spell a border refuses the whole config without a
    /// message -- which is what upstream's `goto fail` does here too.
    fn winborder(&self) -> Result<Parsed, Error> {
        if self.on_a_float() {
            return Ok(Parsed::Done);
        }
        // SAFETY: the option's value is a live NUL-terminated string.
        if p_winborder(CStr::is_empty) {
            return Ok(Parsed::Done);
        }
        // SAFETY: as above, and `self.fconfig` names the live config.
        let parsed = p_winborder(|value| unsafe {
            parse_winborder(self.fconfig.raw(), value.as_ptr().cast_mut())
        })?;
        Ok(if parsed {
            Parsed::Done
        } else {
            Parsed::Refused
        })
    }

    /// The `style` key: `"minimal"`, or nothing at all.
    fn style(&mut self) -> Result<(), Error> {
        let Some(style) = self.config.style.as_ref() else {
            return Ok(());
        };
        let spelling = style.as_cstr();
        self.fconfig.style = if spelling.is_empty() {
            kWinStyleUnused
        } else if imatch(spelling, &[c"minimal"]).is_some() {
            kWinStyleMinimal
        } else {
            return Err(err_invalid_str(c"style", style));
        };
        Ok(())
    }

    /// The keys that are a plain copy: `noautocmd`, `fixed`, `hide` and the
    /// command line's offset.
    fn flags(&mut self) -> Result<(), Error> {
        if let Some(noautocmd) = self.config.noautocmd {
            if self.window.is_some() && noautocmd != self.fconfig.noautocmd {
                let why = c"'noautocmd' cannot be changed on existing window";
                return Err(Error::validation(why));
            }
            self.fconfig.noautocmd = noautocmd;
        }
        if let Some(fixed) = self.config.fixed {
            self.fconfig.fixed = fixed;
        }
        if let Some(hide) = self.config.hide {
            self.fconfig.hide = hide;
        }
        if let Some(offset) = self.config._cmdline_offset {
            self.fconfig._cmdline_offset = offset as c_int;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Integer;

    /// Every name the `relative` key takes, and nothing else. The table and
    /// the values it indexes are two literals that have to stay the same
    /// length, which is the one thing a lookup by index can get wrong.
    #[test]
    #[cfg_attr(miri, ignore = "strcasecmp is a foreign function")]
    fn relative_names_its_six_kinds() {
        let named = [
            (c"editor", kFloatRelativeEditor),
            (c"win", kFloatRelativeWindow),
            (c"cursor", kFloatRelativeCursor),
            (c"mouse", kFloatRelativeMouse),
            (c"tabline", kFloatRelativeTabline),
            (c"laststatus", kFloatRelativeLaststatus),
        ];
        for (name, kind) in named {
            assert_eq!(float_relative(name), Some(kind), "{name:?}");
        }
        assert_eq!(float_relative(c""), None);
        assert_eq!(float_relative(c"editorial"), None);
    }

    /// The anchor is two bits, and its default corner is neither of them.
    #[test]
    #[cfg_attr(miri, ignore = "strcasecmp is a foreign function")]
    fn the_anchor_is_a_corner_of_two_bits() {
        assert_eq!(float_anchor(c"NW"), Some(0));
        assert_eq!(float_anchor(c"NE"), Some(kFloatAnchorEast));
        assert_eq!(float_anchor(c"SW"), Some(kFloatAnchorSouth));
        assert_eq!(
            float_anchor(c"SE"),
            Some(kFloatAnchorSouth | kFloatAnchorEast)
        );
        assert_eq!(float_anchor(c""), None);
    }

    /// The enumerated keys fold case, because `striequal` does.
    #[test]
    #[cfg_attr(miri, ignore = "strcasecmp is a foreign function")]
    fn a_name_is_matched_whatever_its_case() {
        assert_eq!(float_anchor(c"nw"), Some(0));
        assert_eq!(float_relative(c"EDITOR"), Some(kFloatRelativeEditor));
        assert_eq!(config_split(c"Above"), Some(kWinSplitAbove));
    }

    /// The border text's position is the one that does not: upstream reads
    /// it with `strequal`.
    #[test]
    fn the_border_text_position_is_case_sensitive() {
        assert_eq!(align_pos(c"left"), Some(kAlignLeft));
        assert_eq!(align_pos(c"center"), Some(kAlignCenter));
        assert_eq!(align_pos(c"right"), Some(kAlignRight));
        assert_eq!(align_pos(c"Left"), None);
        assert_eq!(align_pos(c""), None);
    }

    #[test]
    #[cfg_attr(miri, ignore = "strcasecmp is a foreign function")]
    fn split_names_its_four_sides() {
        assert_eq!(config_split(c"left"), Some(kWinSplitLeft));
        assert_eq!(config_split(c"right"), Some(kWinSplitRight));
        assert_eq!(config_split(c"above"), Some(kWinSplitAbove));
        assert_eq!(config_split(c"below"), Some(kWinSplitBelow));
        assert_eq!(config_split(c"beside"), None);
    }

    /// `bufpos` is exactly two integers: one is not a position and three is
    /// not either.
    #[test]
    fn bufpos_is_two_integers_or_nothing() {
        let at = float_bufpos(&Array::from(vec![
            Object::Integer(3 as Integer),
            Object::Integer(7 as Integer),
        ]))
        .expect("a pair");
        assert_eq!((at.lnum, at.col), (3, 7));
        assert!(float_bufpos(&Array::EMPTY).is_none());
        assert!(float_bufpos(&Array::from(vec![Object::Integer(3)])).is_none());
        assert!(
            float_bufpos(&Array::from(vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(3),
            ]))
            .is_none()
        );
        assert!(
            float_bufpos(&Array::from(vec![
                Object::Integer(1),
                Object::Boolean(true),
            ]))
            .is_none()
        );
    }
}
