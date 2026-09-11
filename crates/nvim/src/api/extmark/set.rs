//! `nvim_buf_set_extmark()`: placing a mark and its decoration.
//!
//! The keyset this takes is the whole decoration surface: an id, an end
//! position, a highlight (or a list of them), a sign, virtual text with a
//! position and a highlight mode, virtual lines, conceal, spell, a url, a
//! priority and the gravity of both ends.  Each key is validated and packed
//! into a [`Decoration`], which then either goes to the redraw that asked
//! for it (`ephemeral`) or is stored in the buffer as a mark.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::validate::{
    Bad, err_bad_number, err_bad_value, err_expected, err_invalid, err_out_of_range,
};
use crate::decoration::DecorStateRef;
use crate::kvec::Kvec;
use crate::winlayer::{Buf, Live};

/// The keyset this call was handed, with checked field access: the pointer the
/// dispatcher passes stays live for the whole call, so one promise at the head
/// buys every `opts.field` below.
type Opts = Live<KeyDict_set_extmark>;

/// The largest `priority` and `_subpriority` a mark may carry.
const MAX_PRIORITY: Integer = 65535;

/// The decoration one call builds, before anything has taken it over.
///
/// Every allocation below is this call's until the mark or the redraw's
/// decor state takes it; a refusal on the way releases them with
/// [`Decoration::release`].
struct Decoration {
    hl: DecorHighlightInline,
    sign: DecorSignHighlight,
    virt_text: DecorVirtText,
    virt_lines: DecorVirtText,
    /// The `url` key's copy, or null.
    url: *mut ::core::ffi::c_char,
    /// Whether anything went into `hl`.
    has_hl: bool,
    /// Whether `hl_group` named more than one group, which needs a chain of
    /// its own beside the inline highlight.
    has_hl_multiple: bool,
}

impl Decoration {
    /// Nothing set: what every key below writes into.
    fn new() -> Decoration {
        let empty = DecorVirtText {
            flags: 0,
            hl_mode: kHlModeUnknown as uint8_t,
            priority: DECOR_PRIORITY_BASE as DecorPriority,
            width: 0,
            col: 0,
            pos: kVPosEndOfLine,
            data: DecorVirtText_data::Text(VirtText {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut(),
            }),
            next: ::core::ptr::null_mut(),
        };
        Decoration {
            hl: DECOR_HIGHLIGHT_INLINE_INIT,
            sign: DECOR_SIGN_HIGHLIGHT_INIT,
            virt_text: empty,
            virt_lines: DecorVirtText {
                flags: kVTIsLines as uint8_t,
                data: DecorVirtText_data::Lines(VirtLines {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut(),
                }),
                ..empty
            },
            url: ::core::ptr::null_mut(),
            has_hl: false,
            has_hl_multiple: false,
        }
    }

    /// Release what was built, for a refusal that handed it to nobody.
    fn release(&mut self) {
        // SAFETY: both vectors and the url copy are this call's own, and
        // nothing has taken them over.
        unsafe {
            clear_virttext(&raw mut *self.virt_text.data.text_mut());
            clear_virtlines(&raw mut *self.virt_lines.data.lines_mut());
            if !self.url.is_null() {
                xfree(self.url.cast::<::core::ffi::c_void>());
            }
        }
    }

    /// The `hl_group` key: one group, or a stack of them where only the
    /// first is the inline highlight.
    fn highlight_group(&mut self, opts: &Opts) -> Result<(), Error> {
        let Some(given) = opts.hl_group.as_ref() else {
            return Ok(());
        };
        match given.as_array() {
            // SAFETY: the name is a NUL-terminated literal.
            None => self.hl.hl_id = unsafe { object_to_hl_id(given, c"hl_group".as_ptr()) }?,
            Some(groups) => {
                for (n, group) in groups.iter().enumerate() {
                    // SAFETY: as above.
                    let hl_id = unsafe { object_to_hl_id(group, c"hl_group item".as_ptr()) }?;
                    if n == 0 {
                        self.hl.hl_id = hl_id;
                    } else if hl_id != 0 {
                        self.has_hl_multiple = true;
                    }
                }
            }
        }
        self.has_hl = self.hl.hl_id > 0;
        Ok(())
    }

    /// The four sign highlight keys, any of which makes this a sign.
    fn sign_highlights(&mut self, opts: &Opts) {
        self.sign.hl_id = opts.sign_hl_group.unwrap_or(0) as ::core::ffi::c_int;
        self.sign.cursorline_hl_id = opts.cursorline_hl_group.unwrap_or(0) as ::core::ffi::c_int;
        self.sign.number_hl_id = opts.number_hl_group.unwrap_or(0) as ::core::ffi::c_int;
        self.sign.line_hl_id = opts.line_hl_group.unwrap_or(0) as ::core::ffi::c_int;
        if self.sign.hl_id != 0
            || self.sign.cursorline_hl_id != 0
            || self.sign.number_hl_id != 0
            || self.sign.line_hl_id != 0
        {
            self.sign.flags |= kSHIsSign as uint16_t;
        }
    }

    /// The `conceal` and `conceal_lines` keys.
    fn conceal(&mut self, opts: &Opts) -> Result<(), Error> {
        if let Some(conceal) = opts.conceal.as_ref() {
            self.hl.flags |= kSHConceal as uint16_t;
            self.has_hl = true;
            if !conceal.is_empty() {
                let mut ch: ::core::ffi::c_int = 0;
                // SAFETY: the keyset's own NUL-terminated bytes.
                self.hl.conceal_char = unsafe { utfc_ptr2schar(conceal.data(), &raw mut ch) };
                if self.hl.conceal_char == 0 || !vim_isprintc(ch) {
                    return Err(Error::validation(c"conceal char has to be printable"));
                }
            }
        }
        if let Some(conceal_lines) = opts.conceal_lines.as_ref() {
            self.hl.flags |= kSHConcealLines as uint16_t;
            self.has_hl = true;
            // SAFETY: as above.
            if !conceal_lines.is_empty() && unsafe { *conceal_lines.data() } != 0 {
                return Err(Error::validation(
                    c"conceal_lines has to be an empty string",
                ));
            }
        }
        Ok(())
    }

    /// The `virt_text` key, where it sits, and how it blends.
    fn virtual_text(&mut self, opts: &Opts) -> Result<(), Error> {
        if let Some(given) = opts.virt_text.as_ref() {
            let mut width = 0;
            let text = parse_virt_text(given, Some(&mut width))?;
            self.virt_text.width = width;
            *self.virt_text.data.text_mut() = text;
        }
        if let Some(pos) = opts.virt_text_pos.as_ref() {
            self.virt_text.pos = match pos.as_cstr().to_bytes() {
                b"eol" => kVPosEndOfLine,
                b"overlay" => kVPosOverlay,
                b"right_align" => kVPosRightAlign,
                b"eol_right_align" => kVPosEndOfLineRightAlign,
                b"inline" => kVPosInline,
                _ => return Err(err_bad_value(c"virt_text_pos", pos.as_cstr())),
            };
        }
        if let Some(win_col) = opts.virt_text_win_col {
            self.virt_text.col = win_col as ::core::ffi::c_int;
            self.virt_text.pos = kVPosWinCol;
        }
        if opts.hl_eol.unwrap_or(false) {
            self.hl.flags |= kSHHlEol as uint16_t;
        }
        if opts.virt_text_hide.unwrap_or(false) {
            self.virt_text.flags |= kVTHide as uint8_t;
        }
        if opts.virt_text_repeat_linebreak.unwrap_or(false) {
            self.virt_text.flags |= kVTRepeatLinebreak as uint8_t;
        }
        if let Some(mode) = opts.hl_mode.as_ref() {
            self.virt_text.hl_mode = match mode.as_cstr().to_bytes() {
                b"replace" => kHlModeReplace as uint8_t,
                b"combine" => kHlModeCombine as uint8_t,
                b"blend" => {
                    if self.virt_text.pos == kVPosInline {
                        let why = c"cannot use 'blend' hl_mode with inline virtual text";
                        return Err(Error::validation(why));
                    }
                    kHlModeBlend as uint8_t
                }
                _ => return Err(err_bad_value(c"hl_mode", mode.as_cstr())),
            };
        }
        Ok(())
    }

    /// The `virt_lines` key and the three that shape it.
    fn virtual_lines(&mut self, opts: &Opts) -> Result<(), Error> {
        let mut flags: ::core::ffi::c_int = if opts.virt_lines_leftcol.unwrap_or(false) {
            kVLLeftcol as ::core::ffi::c_int
        } else {
            0
        };
        if let Some(overflow) = opts.virt_lines_overflow.as_ref() {
            match overflow.as_cstr().to_bytes() {
                b"scroll" => flags |= kVLScroll as ::core::ffi::c_int,
                b"trunc" => {}
                _ => return Err(err_bad_value(c"virt_lines_overflow", overflow.as_cstr())),
            }
        }
        if let Some(given) = opts.virt_lines.as_ref() {
            for item in given.iter() {
                let Object::Array(chunks) = item else {
                    let want = api_typename(kObjectTypeArray);
                    let got = api_typename(item.kind());
                    return Err(err_expected(c"virt_text_line", want, Some(got)));
                };
                let line = virt_line {
                    line: parse_virt_text(chunks, None)?,
                    flags,
                };
                // `kv_push`, whose growth step c2rust expanded inline.
                let lines = self.virt_lines.data.lines_mut();
                let mut vl = Kvec::new(&mut lines.size, &mut lines.capacity, &mut lines.items);
                // SAFETY: `items` is this vector's own allocation.
                unsafe { vl.push(line) };
            }
        }
        if opts.virt_lines_above.unwrap_or(false) {
            self.virt_lines.flags |= kVTLinesAbove as uint8_t;
        }
        Ok(())
    }

    /// The `priority` key, which every part of the decoration shares.
    fn priority(&mut self, opts: &Opts) -> Result<(), Error> {
        let Some(given) = opts.priority else {
            return Ok(());
        };
        if !(0..=MAX_PRIORITY).contains(&given) {
            return Err(err_out_of_range(c"priority"));
        }
        let priority = given as DecorPriority;
        self.hl.priority = priority;
        self.sign.priority = priority;
        self.virt_text.priority = priority;
        self.virt_lines.priority = priority;
        Ok(())
    }

    /// The `sign_text` key: one or two cells of the sign column.
    fn sign_text(&mut self, opts: &Opts) -> Result<(), Error> {
        let Some(given) = opts.sign_text.as_ref() else {
            return Ok(());
        };
        self.sign.text[0] = 0;
        let into = (&raw mut self.sign.text).cast::<ScreenChar>();
        // SAFETY: the keyset's own NUL-terminated bytes, into this call's own
        // two-cell array.
        if unsafe { init_sign_text(given.data(), into, false) }.is_err() {
            return Err(err_invalid(c"sign_text", Bad::Unsaid));
        }
        self.sign.flags |= kSHIsSign as uint16_t;
        Ok(())
    }

    /// The `spell`, `url` and `ui_watched` keys, each of which is a
    /// highlight even when no group was named.
    fn highlight_extras(&mut self, opts: &Opts) {
        if let Some(spell) = opts.spell {
            self.hl.flags |= (if spell { kSHSpellOn } else { kSHSpellOff }) as uint16_t;
            self.has_hl = true;
        }
        if let Some(given) = opts.url.as_ref() {
            self.url = string_to_cstr(given);
            self.has_hl = true;
        }
        if opts.ui_watched.unwrap_or(false) {
            self.hl.flags |= kSHUIWatched as uint16_t;
            if self.virt_text.pos == kVPosOverlay {
                self.hl.flags |= kSHUIWatchedOverlay as uint16_t;
            }
            self.has_hl = true;
        }
    }

    /// Hand the decoration to the redraw that asked for it, rather than
    /// storing a mark: what `ephemeral` means.  `state` is the redraw's own,
    /// which is what makes the ranges below the right ones to add to.
    fn add_to_redraw(
        &mut self,
        state: DecorStateRef,
        at: Range,
        opts: &Opts,
        ns_id: Integer,
        id: uint32_t,
    ) -> Result<(), Error> {
        // An ephemeral mark with no end covers the one position it names.
        let (row, col) = (at.line as ::core::ffi::c_int, at.col as ColNr);
        let (end_row, end_col) = if at.line2 == -1 {
            (row, col)
        } else {
            (at.line2, at.col2)
        };
        let mut subpriority: DecorPriority = 0;
        if let Some(given) = opts._subpriority {
            if !(0..=MAX_PRIORITY).contains(&given) {
                return Err(err_out_of_range(c"_subpriority"));
            }
            subpriority = given as DecorPriority;
        }
        // SAFETY: `state` is the redraw's own, and each part of the
        // decoration is handed over exactly once.
        unsafe {
            if self.virt_text.data.text().size != 0 {
                let vt = decor_put_vt(self.virt_text, ::core::ptr::null_mut());
                decor_range_add_virt(state, row, col, end_row, end_col, vt, true);
            }
            if self.virt_lines.data.lines().size != 0 {
                let vt = decor_put_vt(self.virt_lines, ::core::ptr::null_mut());
                decor_range_add_virt(state, row, col, end_row, end_col, vt, true);
            }
            if self.has_hl {
                let mut sh: DecorSignHighlight = decor_sh_from_inline(self.hl);
                sh.url = self.url;
                decor_range_add_sh(
                    state,
                    row,
                    col,
                    end_row,
                    end_col,
                    &raw mut sh,
                    true,
                    ns_id as uint32_t,
                    id,
                    subpriority,
                );
            }
        }
        Ok(())
    }

    /// Store the mark, packing the decoration inline where it fits and into
    /// the allocated chains where it does not.
    fn store(
        &mut self,
        buffer: Buf,
        at: Range,
        opts: &Opts,
        ns_id: Integer,
        id: &mut uint32_t,
        right_gravity: bool,
    ) {
        let mut decor_flags = MtFlags::NONE;
        let mut decor_alloc: *mut DecorVirtText = ::core::ptr::null_mut();
        if self.virt_text.data.text().size != 0 {
            decor_alloc = decor_put_vt(self.virt_text, decor_alloc);
            if self.virt_text.pos == kVPosInline {
                decor_flags |= MtFlags::DECOR_VIRT_TEXT_INLINE;
            }
        }
        if self.virt_lines.data.lines().size != 0 {
            decor_alloc = decor_put_vt(self.virt_lines, decor_alloc);
            decor_flags |= MtFlags::DECOR_VIRT_LINES;
        }
        let mut decor_indexed: uint32_t = DECOR_ID_INVALID;
        if self.sign.flags & kSHIsSign as uint16_t != 0 {
            self.sign.next = decor_indexed;
            decor_indexed = decor_put_sh(self.sign);
            if self.sign.text[0] != 0 {
                decor_flags |= MtFlags::DECOR_SIGNTEXT;
            }
            if self.sign.number_hl_id != 0
                || self.sign.line_hl_id != 0
                || self.sign.cursorline_hl_id != 0
            {
                decor_flags |= MtFlags::DECOR_SIGNHL;
            }
        }
        if self.has_hl_multiple {
            let Some(groups) = opts.hl_group.as_ref().and_then(Object::as_array) else {
                unreachable!("`has_hl_multiple` is set only under an Array")
            };
            // Backwards, and without the first: the chain is read from its
            // tail, and group zero is the inline highlight.
            for group in groups.iter().skip(1).rev() {
                // The same objects resolved above, so a refusal here is
                // impossible; zero is the id an unresolvable name would have
                // got.
                // SAFETY: the name is a NUL-terminated literal.
                let hl_id = unsafe { object_to_hl_id(group, c"hl_group item".as_ptr()) };
                let hl_id = hl_id.unwrap_or(0);
                if hl_id > 0 {
                    let mut sh: DecorSignHighlight = DECOR_SIGN_HIGHLIGHT_INIT;
                    sh.hl_id = hl_id;
                    sh.flags = if opts.hl_eol.unwrap_or(false) {
                        kSHHlEol as uint16_t
                    } else {
                        0
                    };
                    sh.next = decor_indexed;
                    decor_indexed = decor_put_sh(sh);
                    decor_flags |= MtFlags::DECOR_HL;
                }
            }
        }
        if self.hl.flags & kSHConcealLines as uint16_t != 0 {
            decor_flags |= MtFlags::DECOR_CONCEAL_LINES;
        }
        let mut decor: DecorInline = DECOR_INLINE_INIT;
        if !decor_alloc.is_null()
            || decor_indexed != DECOR_ID_INVALID
            || !self.url.is_null()
            || schar_high(self.hl.conceal_char)
        {
            if self.has_hl {
                let mut sh: DecorSignHighlight = decor_sh_from_inline(self.hl);
                sh.url = self.url;
                sh.next = decor_indexed;
                decor_indexed = decor_put_sh(sh);
            }
            decor.ext = true;
            decor.data.ext = DecorExt {
                sh_idx: decor_indexed,
                vt: decor_alloc,
            };
        } else {
            decor.data.hl = self.hl;
        }
        if self.has_hl {
            decor_flags |= MtFlags::DECOR_HL;
        }
        // SAFETY: the caller's buffer, and a decoration this call owns until
        // the mark takes it over.
        unsafe {
            extmark_set(
                buffer,
                ns_id as uint32_t,
                id,
                at.line as ::core::ffi::c_int,
                at.col as ColNr,
                at.line2,
                at.col2,
                decor,
                decor_flags,
                right_gravity,
                opts.end_right_gravity.unwrap_or(false),
                !opts.undo_restore.unwrap_or(true),
                opts.invalidate.unwrap_or(false),
            );
        }
    }
}

/// Where the mark goes, once both ends have been clamped against the buffer.
///
/// `line2`/`col2` are `-1` for a mark with no end.
struct Range {
    line: Integer,
    col: Integer,
    line2: ::core::ffi::c_int,
    col2: ColNr,
}

/// # Safety
///
/// `opts` must point at the `KeyDict_set_extmark` the dispatcher filled in,
/// live for the call.
pub unsafe fn nvim_buf_set_extmark(
    buf: BufferHandle,
    ns_id: Integer,
    line: Integer,
    col: Integer,
    opts: *mut KeyDict_set_extmark,
) -> Result<Integer, Error> {
    // SAFETY: the dispatcher's keyset outlives this call.
    let mut opts = unsafe { Opts::new(opts) };
    let Some(b) = find_buffer_by_handle(buf)? else {
        return Ok(0);
    };
    if !ns_initialized(ns_id as uint32_t) {
        return Err(err_bad_number(c"ns_id", ns_id));
    }
    let mut decor = Decoration::new();
    match set_extmark(&mut decor, b, ns_id, line, col, &mut opts) {
        Ok(id) => Ok(id),
        Err(refused) => {
            // Nothing took the half-built decoration over.
            decor.release();
            Err(refused)
        }
    }
}

/// Read every key into `decor`, clamp the positions, and place the mark.
///
/// A refusal leaves `decor` for the caller to release: no key that can refuse
/// runs after the first thing the redraw or the mark takes over.
fn set_extmark(
    decor: &mut Decoration,
    b: Buf,
    ns_id: Integer,
    line: Integer,
    col: Integer,
    opts: &mut Opts,
) -> Result<Integer, Error> {
    let mut id: uint32_t = 0;
    if let Some(given) = opts.id {
        if given <= 0 {
            return Err(err_expected(c"id", c"positive Integer", None));
        }
        id = given as uint32_t;
    }
    let strict = opts.strict.unwrap_or(true);
    let (line2, col2) = end_position(opts, b, strict)?;
    decor.highlight_group(opts)?;
    decor.sign_highlights(opts);
    decor.conceal(opts)?;
    decor.virtual_text(opts)?;
    decor.virtual_lines(opts)?;
    decor.priority(opts)?;
    decor.sign_text(opts)?;
    let right_gravity = opts.right_gravity.unwrap_or(true);
    if line2 == -1 && col2 == -1 && opts.end_right_gravity.is_some() {
        let why = c"cannot set end_right_gravity without end_row or end_col";
        return Err(Error::validation(why));
    }
    decor.highlight_extras(opts);

    let ephemeral = opts.ephemeral.unwrap_or(false);
    let at = clamp(b, line, col, line2, col2, strict, ephemeral)?;

    if ephemeral {
        // SAFETY: nothing here holds a `decor_state` borrow.
        let state = unsafe { DecorStateRef::current() };
        // A provider is running exactly when the redraw has a window, and it
        // has to be drawing this buffer.
        // SAFETY: a non-null window of the state is a live one.
        let drawing = !state.win.is_null() && unsafe { (*state.win).w_buffer } == b.raw();
        if !drawing {
            let why = c"cannot set emphemeral mark outside of a decoration provider";
            return Err(Error::exception(why));
        }
        decor.add_to_redraw(state, at, opts, ns_id, id)?;
    } else {
        decor.store(b, at, opts, ns_id, &mut id, right_gravity);
    }
    Ok(Integer::from(id))
}

/// The `end_row`/`end_line`/`end_col` keys, as the mark's end position.
///
/// `(-1, -1)` is a mark with no end at all.
fn end_position(
    opts: &mut Opts,
    b: Buf,
    strict: bool,
) -> Result<(::core::ffi::c_int, ColNr), Error> {
    if let Some(end_line) = opts.end_line {
        if opts.end_row.is_some() {
            return Err(Error::validation(
                c"cannot use both 'end_row' and 'end_line'",
            ));
        }
        opts.end_row = Some(end_line);
    }
    let mut line2 = -1;
    // `end_line` wrote `end_row` above, so one test covers both.
    if let Some(given) = opts.end_row {
        if given < 0 || (given > b.line_count() as Integer && strict) {
            return Err(err_out_of_range(c"end_row"));
        }
        line2 = given as ::core::ffi::c_int;
    }
    let mut col2: ColNr = -1;
    if let Some(given) = opts.end_col {
        if !(-1..=Integer::from(MAXCOL)).contains(&given) {
            return Err(err_out_of_range(c"end_col"));
        }
        // `-1` is "to the end of the line", which is what `MAXCOL` says.
        col2 = if given == -1 { MAXCOL } else { given as ColNr };
    }
    Ok((line2, col2))
}

/// Both ends of the mark against the buffer's lines.
///
/// `strict` refuses a position past the end rather than pulling it back; an
/// `ephemeral` mark measures every line as `MAXCOL`, because the redraw is
/// mid-change and the stored lengths are not what is being drawn.
///
fn clamp(
    b: Buf,
    mut line: Integer,
    mut col: Integer,
    mut line2: ::core::ffi::c_int,
    mut col2: ColNr,
    strict: bool,
    ephemeral: bool,
) -> Result<Range, Error> {
    // SAFETY: `at` is a line of the buffer, which every caller below has
    // just tested against its line count.
    let line_len = |at: LineNr| -> ColNr {
        if ephemeral {
            MAXCOL
        } else {
            unsafe { b.line_len(at) }
        }
    };
    if line < 0 {
        return Err(err_out_of_range(c"line"));
    }
    let mut len: ColNr = 0;
    if line > b.line_count() as Integer {
        if strict {
            return Err(err_out_of_range(c"line"));
        }
        line = b.line_count() as Integer;
    } else if line < b.line_count() as Integer {
        len = line_len(line as LineNr + 1);
    }
    if col == -1 {
        col = Integer::from(len);
    } else if col > Integer::from(len) {
        if strict {
            return Err(err_out_of_range(c"col"));
        }
        col = Integer::from(len);
    } else if col < -1 {
        return Err(err_out_of_range(c"col"));
    }
    if col2 >= 0 {
        if line2 >= 0 && (line2 as LineNr) < b.line_count() {
            len = line_len(line2 as LineNr + 1);
        } else if line2 as LineNr == b.line_count() {
            len = 0;
        } else {
            // An end column with no end row is on the mark's own line.
            line2 = line as ::core::ffi::c_int;
        }
        if col2 > len {
            if strict {
                return Err(err_out_of_range(c"end_col"));
            }
            col2 = len;
        }
    } else if line2 >= 0 {
        col2 = 0;
    }
    Ok(Range {
        line,
        col,
        line2,
        col2,
    })
}
