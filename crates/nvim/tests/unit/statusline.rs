//! `'statusline'` rendering, end to end.
//!
//! A port of `test/unit/statusline_spec.lua`. Everything here goes through
//! [`build_stl_str_hl`], the one entry point that turns a format string into
//! the bytes a status line is drawn from, because that is where the parts
//! that have no other home meet: the `%=` separators dividing what is left
//! over, the truncation marks (`<`, `>`, and the `%<` the format may place
//! itself), the fill character between aligned groups, and the item groups
//! with their own widths.
//!
//! The tables are the spec's, extracted from its Lua rather than retyped —
//! a row lost in transcription is a silent weakening, and there are 123 of
//! them. Each row is `(cells available, format, expected text)` plus, where
//! it differs, the cell count the call should answer.
//!
//! The editor is needed throughout: the format may be an expression (`%!`),
//! the items read `curwin` and `curbuf`, and the fill character is interned
//! in the process-wide glyph cache.

#![cfg(not(miri))]

use std::ffi::{CStr, c_char, c_int};
use std::ptr;

use neovim::buffer::setfname;
use neovim::grid::schar_from_str;
use neovim::main::{curbuf, curwin};
use neovim::statusline::{FmtSource, StlSinks, build_stl_str_hl};
use neovim::types::schar_T;

use crate::support::{Sandbox, cstr};

/// The output buffer the spec used, in bytes. Every expectation is shorter
/// than this, so nothing here is testing what happens when the buffer
/// itself runs out — `maxwidth` is the limit under test.
const BUF_BYTES: usize = 120;

/// One row of a table: render `fmt` into `cells` cells and expect `want`.
#[derive(Clone, Copy)]
struct Case {
    /// `maxwidth`: how many screen cells the status line may occupy.
    cells: c_int,
    fmt: &'static str,
    want: &'static str,
    /// The cell count the call should answer, when it is not `cells` — a
    /// format that does not fill the width answers what it used.
    want_cells: Option<c_int>,
    /// The buffer's file name, or `""` for a buffer that has none.
    file: &'static str,
    /// The fill character, as the text it is interned from. `""` is the
    /// glyph zero, which the renderer reads as a space.
    fill: &'static str,
    /// Run this row three times, once per fill character, substituting each
    /// for the `~`s in `want`. Alignment is where the fill character is
    /// visible, and where a byte-vs-cell confusion shows up: `━` is three
    /// bytes wide and one cell.
    aligned: bool,
}

/// A row run once, with the default fill character.
const PLAIN: Case = Case {
    cells: 0,
    fmt: "",
    want: "",
    want_cells: None,
    file: "",
    fill: " ",
    aligned: false,
};

/// A row run once per fill character, with `~` standing in for it.
const ALIGNED: Case = Case {
    aligned: true,
    ..PLAIN
};

/// The fill characters an [`ALIGNED`] row is run with: the default, a
/// single-byte one, and a multibyte one.
const FILLS: [&str; 3] = [" ", "!", "━"];

/// The editor, plus the promise that `curbuf`'s file name is put back.
///
/// The spec's `itp` forked a child per case, so naming the current buffer
/// was free; here every case in the process shares it.
struct Statusline {
    _sandbox: Sandbox,
}

impl Statusline {
    fn claim() -> Statusline {
        Statusline {
            _sandbox: Sandbox::globals(),
        }
    }

    /// Name `curbuf`, or clear its name when `file` is empty.
    fn name(&self, file: &str) {
        let name = cstr(file);
        let ptr = if file.is_empty() {
            ptr::null_mut()
        } else {
            name.as_ptr().cast_mut()
        };
        // SAFETY: `curbuf` is the editor's own buffer under the editor
        // lock, and `name` outlives the call, which copies what it keeps.
        unsafe { setfname(curbuf.get(), ptr, ptr::null_mut(), true) };
    }

    /// Render one row and check both halves of the answer: the bytes
    /// written, and the cell count returned.
    fn check(&self, case: &Case, want: &str, fill: &str) {
        self.check_owned(case, case.fmt, want, fill);
    }

    /// [`Statusline::check`] with the format supplied separately, for the
    /// rows whose format is built at run time.
    fn check_owned(&self, case: &Case, format: &str, want: &str, fill: &str) {
        self.name(case.file);
        let mut out = vec![b' ' as c_char; BUF_BYTES];
        let fmt = cstr(format);
        let fill_str = cstr(fill);
        // SAFETY: `fill_str` is NUL-terminated.
        let fillchar: schar_T = if fill.is_empty() {
            0
        } else {
            unsafe { schar_from_str(fill_str.as_ptr()) }
        };
        // SAFETY: `fmt` is NUL-terminated and outlives the call, `curwin`
        // is the editor's window, and no sink is asked for.
        let cells = unsafe {
            build_stl_str_hl(
                curwin.get(),
                &mut out,
                fmt.as_ptr().cast_mut(),
                FmtSource::NONE,
                fillchar,
                case.cells,
                StlSinks::NONE,
            )
        };
        // The whole rendering, not a prefix of it: the spec compared only
        // the first `#expected` bytes, so a renderer that wrote *more* than
        // it should went unnoticed.
        // SAFETY: `build_stl_str_hl` NUL-terminates within `BUF_BYTES`.
        let got = unsafe { CStr::from_ptr(out.as_ptr().cast::<c_char>()) };
        let got = String::from_utf8_lossy(got.to_bytes());
        let what = format!("/{format}/ in {} cells, fill {fill:?}", case.cells);
        assert_eq!(got, want, "{what}");
        assert_eq!(
            cells,
            case.want_cells.unwrap_or(case.cells),
            "cells of {what}"
        );
    }
}

impl Drop for Statusline {
    fn drop(&mut self) {
        self.name("");
    }
}

/// Run a table, expanding every [`ALIGNED`] row over [`FILLS`].
fn run(cases: &[Case]) {
    let stl = Statusline::claim();
    for case in cases {
        if case.aligned {
            for fill in FILLS {
                stl.check(case, &case.want.replace('~', fill), fill);
            }
        } else {
            stl.check(case, case.want, case.fill);
        }
    }
}

/// `%!expr` evaluates `expr` and renders *that* as the format. A broken
/// expression is not an error: the literal text is what gets rendered.
#[test]
fn a_bang_format_is_an_expression_whose_result_is_the_format() {
    run(&[
        // Should expand expression
        Case {
            cells: 2,
            fmt: "%!expand(20+1)",
            want: "21",
            ..PLAIN
        },
        // Should expand broken expression to itself
        Case {
            cells: 11,
            fmt: "%!expand(20+1",
            want: "expand(20+1",
            ..PLAIN
        },
    ]);
}

/// `%f` is the name as the user would type it, `%F` the full path, `%t`
/// the last component; a buffer with no name renders `[No Name]`.
///
/// The paths are names, not files: `setfname` records what it is given
/// and nothing here opens anything.
#[test]
fn the_file_name_items_name_the_buffer_three_ways() {
    run(&[
        // should print no file name
        Case {
            cells: 10,
            fmt: "%f",
            want: "[No Name]",
            want_cells: Some(9),
            ..PLAIN
        },
        // should print the relative file name
        Case {
            cells: 30,
            fmt: "%f",
            want: "test/unit/buffer_spec.lua",
            want_cells: Some(25),
            file: "test/unit/buffer_spec.lua",
            ..PLAIN
        },
        // should print the full file name
        Case {
            cells: 40,
            fmt: "%F",
            want: "/test/unit/buffer_spec.lua",
            want_cells: Some(26),
            file: "/test/unit/buffer_spec.lua",
            ..PLAIN
        },
        // should print the tail file name
        Case {
            cells: 80,
            fmt: "%t",
            want: "buffer_spec.lua",
            want_cells: Some(15),
            file: "test/unit/buffer_spec.lua",
            ..PLAIN
        },
    ]);
}

/// Whatever `%=` leaves over is filled with the caller's fill character,
/// which may be multibyte. The glyph zero means "a space".
#[test]
fn the_fill_character_fills_what_a_separator_leaves() {
    run(&[
        // should handle `!` as a fillchar
        Case {
            cells: 10,
            fmt: "abcde%=",
            want: "abcde!!!!!",
            fill: "!",
            ..PLAIN
        },
        // should handle `~` as a fillchar
        Case {
            cells: 10,
            fmt: "%=abcde",
            want: "~~~~~abcde",
            fill: "~",
            ..PLAIN
        },
        // should put fillchar `!` in between text
        Case {
            cells: 10,
            fmt: "abc%=def",
            want: "abc!!!!def",
            fill: "!",
            ..PLAIN
        },
        // should put fillchar `~` in between text
        Case {
            cells: 10,
            fmt: "abc%=def",
            want: "abc~~~~def",
            fill: "~",
            ..PLAIN
        },
        // should put fillchar `━` in between text
        Case {
            cells: 10,
            fmt: "abc%=def",
            want: "abc━━━━def",
            fill: "━",
            ..PLAIN
        },
        // should handle zero-fillchar as a space
        Case {
            cells: 10,
            fmt: "abcde%=",
            want: "abcde     ",
            fill: "",
            ..PLAIN
        },
    ]);
}

/// Everything that is not an item is copied out unchanged.
#[test]
fn text_outside_an_item_is_copied_through() {
    run(&[
        // should copy plain text
        Case {
            cells: 80,
            fmt: "this is a test",
            want: "this is a test",
            want_cells: Some(14),
            ..PLAIN
        },
    ]);
}

/// `%n` is the buffer number, `%l` the cursor line and `%L` the line
/// count. The editor's own buffer is empty, so the cursor is on line 0 of
/// one line.
#[test]
fn the_position_items_read_the_current_buffer() {
    run(&[
        // should print the buffer number
        Case {
            cells: 80,
            fmt: "%n",
            want: "1",
            want_cells: Some(1),
            ..PLAIN
        },
        // should print the current line number in the buffer
        Case {
            cells: 80,
            fmt: "%l",
            want: "0",
            want_cells: Some(1),
            ..PLAIN
        },
        // should print the number of lines in the buffer
        Case {
            cells: 80,
            fmt: "%L",
            want: "1",
            want_cells: Some(1),
            ..PLAIN
        },
    ]);
}

/// Text that does not fit loses its middle or its head, and the cut is
/// marked: `<` when the start was dropped, `>` when the end was. `%<` in
/// the format says where the cut should fall, and only the first one
/// counts.
#[test]
fn a_line_too_long_for_its_width_is_truncated_at_a_mark() {
    run(&[
        // should truncate when standard text pattern is too long
        Case {
            cells: 10,
            fmt: "0123456789abcde",
            want: "<6789abcde",
            ..PLAIN
        },
        // should truncate when using =
        Case {
            cells: 10,
            fmt: "abcdef%=ghijkl",
            want: "abcdef<jkl",
            ..PLAIN
        },
        // should truncate centered text when using ==
        Case {
            cells: 10,
            fmt: "abcde%=gone%=fghij",
            want: "abcde<ghij",
            ..PLAIN
        },
        // should respect the `<` marker
        Case {
            cells: 10,
            fmt: "abc%<defghijkl",
            want: "abc<ghijkl",
            ..PLAIN
        },
        // should truncate at `<` with one `=`, test 1
        Case {
            cells: 10,
            fmt: "abc%<def%=ghijklmno",
            want: "abc<jklmno",
            ..PLAIN
        },
        // should truncate at `<` with one `=`, test 2
        Case {
            cells: 10,
            fmt: "abcdef%=ghijkl%<mno",
            want: "abcdefghi>",
            ..PLAIN
        },
        // should truncate at `<` with one `=`, test 3
        Case {
            cells: 10,
            fmt: "abc%<def%=ghijklmno",
            want: "abc<jklmno",
            ..PLAIN
        },
        // should truncate at `<` with one `=`, test 4
        Case {
            cells: 10,
            fmt: "abc%<def%=ghij",
            want: "abcdefghij",
            ..PLAIN
        },
        // should truncate at `<` with one `=`, test 4
        Case {
            cells: 10,
            fmt: "abc%<def%=ghijk",
            want: "abc<fghijk",
            ..PLAIN
        },
        // should truncate at `<` with many `=`, test 4
        Case {
            cells: 10,
            fmt: "ab%<cdef%=g%=h%=ijk",
            want: "ab<efghijk",
            ..PLAIN
        },
        // should truncate at the first `<`
        Case {
            cells: 10,
            fmt: "abc%<def%<ghijklm",
            want: "abc<hijklm",
            ..PLAIN
        },
        // should ignore trailing %
        Case {
            cells: 3,
            fmt: "abc%",
            want: "abc",
            ..PLAIN
        },
    ]);
}

/// A cut that lands inside a double-width character leaves a cell over, and
/// the fill character makes the difference up — after which the line should
/// be exactly `maxwidth` cells and nothing more.
///
/// It is not. `build_stl_str_hl`'s top-up loop appends the fill character
/// *over* the string's terminator and never writes another, so the answer
/// runs on into whatever the output buffer held before —
/// `nvim_eval_statusline('12🙂345', {maxwidth = 5})` reports a width of 5
/// and hands back nine cells of text. Upstream `statusline.c` does the same
/// thing (`while (++width < maxwidth) { schar_get_adv(...); end = trunc_p; }
/// (void)end;`), so this is not something the port introduced and not
/// something to fix from here.
///
/// The spec had a case in this shape but only over an item *group*, whose
/// truncation is different code and does terminate — and it compared only
/// the first `#expected` bytes of the buffer, which cannot see an answer
/// that is too long. Restore this to a plain `#[test]` when the terminator
/// is written.
#[test]
#[ignore = "upstream: the half-cell top-up overwrites the terminator"]
fn a_cut_at_a_double_width_character_is_padded_to_the_full_width() {
    run(&[Case {
        cells: 5,
        fmt: "12\u{1f642}345",
        want: "<345~",
        ..ALIGNED
    }]);
}

/// `%=` is a separator, not an alignment: with one the text either side is
/// pushed apart, with several the leftover cells are divided as evenly as
/// they go, remainder to the right.
#[test]
fn separators_divide_the_leftover_space_evenly() {
    run(&[
        // should right align when using =
        Case {
            cells: 20,
            fmt: "neo%=vim",
            want: "neo~~~~~~~~~~~~~~vim",
            ..ALIGNED
        },
        // should, when possible, center text when using %=text%=
        Case {
            cells: 20,
            fmt: "abc%=neovim%=def",
            want: "abc~~~~neovim~~~~def",
            ..ALIGNED
        },
        // should handle uneven spacing in the buffer when using %=text%=
        Case {
            cells: 20,
            fmt: "abc%=neo_vim%=def",
            want: "abc~~~neo_vim~~~~def",
            ..ALIGNED
        },
        // should have equal spaces even with non-equal sides when using =
        Case {
            cells: 20,
            fmt: "foobar%=test%=baz",
            want: "foobar~~~test~~~~baz",
            ..ALIGNED
        },
        // should have equal spaces even with longer right side when using =
        Case {
            cells: 20,
            fmt: "a%=test%=longtext",
            want: "a~~~test~~~~longtext",
            ..ALIGNED
        },
        // should handle an empty left side when using ==
        Case {
            cells: 20,
            fmt: "%=test%=baz",
            want: "~~~~~~test~~~~~~~baz",
            ..ALIGNED
        },
        // should handle an empty right side when using ==
        Case {
            cells: 20,
            fmt: "foobar%=test%=",
            want: "foobar~~~~~test~~~~~",
            ..ALIGNED
        },
        // should handle consecutive empty ==
        Case {
            cells: 20,
            fmt: "%=%=test%=",
            want: "~~~~~~~~~~test~~~~~~",
            ..ALIGNED
        },
        // should handle an = alone
        Case {
            cells: 20,
            fmt: "%=",
            want: "~~~~~~~~~~~~~~~~~~~~",
            ..ALIGNED
        },
        // should right align text when it is alone with =
        Case {
            cells: 20,
            fmt: "%=foo",
            want: "~~~~~~~~~~~~~~~~~foo",
            ..ALIGNED
        },
        // should left align text when it is alone with =
        Case {
            cells: 20,
            fmt: "foo%=",
            want: "foo~~~~~~~~~~~~~~~~~",
            ..ALIGNED
        },
        // should approximately center text when using %=text%=
        Case {
            cells: 21,
            fmt: "abc%=neovim%=def",
            want: "abc~~~~neovim~~~~~def",
            ..ALIGNED
        },
        // should completely fill the buffer when using %=text%=
        Case {
            cells: 21,
            fmt: "abc%=neo_vim%=def",
            want: "abc~~~~neo_vim~~~~def",
            ..ALIGNED
        },
        // should have equal spacing even with non-equal sides when using =
        Case {
            cells: 21,
            fmt: "foobar%=test%=baz",
            want: "foobar~~~~test~~~~baz",
            ..ALIGNED
        },
        // should have equal spacing even with longer right side when using =
        Case {
            cells: 21,
            fmt: "a%=test%=longtext",
            want: "a~~~~test~~~~longtext",
            ..ALIGNED
        },
        // should handle an empty left side when using ==
        Case {
            cells: 21,
            fmt: "%=test%=baz",
            want: "~~~~~~~test~~~~~~~baz",
            ..ALIGNED
        },
        // should handle an empty right side when using ==
        Case {
            cells: 21,
            fmt: "foobar%=test%=",
            want: "foobar~~~~~test~~~~~~",
            ..ALIGNED
        },
        // should quadrant the text when using 3 %=
        Case {
            cells: 40,
            fmt: "abcd%=n%=eovim%=ef",
            want: "abcd~~~~~~~~~n~~~~~~~~~eovim~~~~~~~~~~ef",
            ..ALIGNED
        },
        // should work well with %t
        Case {
            cells: 40,
            fmt: "%t%=right_aligned",
            want: "buffer_spec.lua~~~~~~~~~~~~right_aligned",
            file: "test/unit/buffer_spec.lua",
            ..ALIGNED
        },
        // should work well with %t and regular text
        Case {
            cells: 40,
            fmt: "l%=m_l %t m_r%=r",
            want: "l~~~~~~~m_l buffer_spec.lua m_r~~~~~~~~r",
            file: "test/unit/buffer_spec.lua",
            ..ALIGNED
        },
        // should work well with %=, %t, %L, and %l
        Case {
            cells: 40,
            fmt: "%t %= %L %= %l",
            want: "buffer_spec.lua ~~~~~~~~~ 1 ~~~~~~~~~~ 0",
            file: "test/unit/buffer_spec.lua",
            ..ALIGNED
        },
        // should quadrant the text when using 3 %=
        Case {
            cells: 41,
            fmt: "abcd%=n%=eovim%=ef",
            want: "abcd~~~~~~~~~n~~~~~~~~~eovim~~~~~~~~~~~ef",
            ..ALIGNED
        },
        // should work well with %t
        Case {
            cells: 41,
            fmt: "%t%=right_aligned",
            want: "buffer_spec.lua~~~~~~~~~~~~~right_aligned",
            file: "test/unit/buffer_spec.lua",
            ..ALIGNED
        },
        // should work well with %t and regular text
        Case {
            cells: 41,
            fmt: "l%=m_l %t m_r%=r",
            want: "l~~~~~~~~m_l buffer_spec.lua m_r~~~~~~~~r",
            file: "test/unit/buffer_spec.lua",
            ..ALIGNED
        },
        // should work well with %=, %t, %L, and %l
        Case {
            cells: 41,
            fmt: "%t %= %L %= %l",
            want: "buffer_spec.lua ~~~~~~~~~~ 1 ~~~~~~~~~~ 0",
            file: "test/unit/buffer_spec.lua",
            ..ALIGNED
        },
        // should work with 10 %=
        Case {
            cells: 50,
            fmt: "aaaa%=b%=c%=d%=e%=fg%=hi%=jk%=lmnop%=qrstuv%=wxyz",
            want: "aaaa~~b~~c~~d~~e~~fg~~hi~~jk~~lmnop~~qrstuv~~~wxyz",
            ..ALIGNED
        },
    ]);
}

/// `%N(...%)` is a group N cells wide, right-aligned inside those cells
/// unless `-` says left; `%N.M(` also truncates the group's contents to M.
/// Truncating in the middle of a multicell character leaves a cell to make
/// up, which the fill character takes.
#[test]
fn an_item_group_carries_its_own_width_and_alignment() {
    run(&[
        // should right align in right aligned groups
        Case {
            cells: 30,
            fmt: "%5(%l%), %5.(%l%), %5.5(%l%), %5.10(%l%)",
            want: "~~~~0, ~~~~0, ~~~~0, ~~~~0",
            want_cells: Some(26),
            ..ALIGNED
        },
        // should left align in left aligned groups
        Case {
            cells: 30,
            fmt: "%-5(%l%), %-5.(%l%), %-5.5(%l%), %-5.10(%l%)",
            want: "0~~~~, 0~~~~, 0~~~~, 0~~~~",
            want_cells: Some(26),
            ..ALIGNED
        },
        // Should truncate groups according to maxwid
        Case {
            cells: 30,
            fmt: "%.5(1234567%), %2.5(1234567%), %5.5(1234567%)",
            want: "<4567, <4567, <4567",
            want_cells: Some(19),
            ..PLAIN
        },
        // Should compensate with fillchar after truncating item group at multicell character
        Case {
            cells: 20,
            fmt: "%.5(12🙂345%), %5.5(12🙂345%), %50.5(12🙂345%)",
            want: "<345, <345~, <345~",
            want_cells: Some(18),
            ..ALIGNED
        },
    ]);
}

/// The width budget is in cells and the buffer is in bytes; a format that
/// confuses them mis-sizes the fill.
#[test]
fn a_multibyte_character_is_one_cell_and_several_bytes() {
    run(&[
        // should handle multibyte characters
        Case {
            cells: 10,
            fmt: "Ĉ%=x",
            want: "Ĉ        x",
            ..PLAIN
        },
        // should handle multibyte characters and different fillchars
        Case {
            cells: 10,
            fmt: "Ą%=mid%=end",
            want: "Ą@mid@@end",
            fill: "@",
            ..PLAIN
        },
    ]);
}

/// `%%` renders one `%`, including when it is the first thing in the
/// format or the thing that does not fit.
#[test]
fn a_doubled_percent_is_a_literal_one() {
    run(&[
        // should handle escape of %
        Case {
            cells: 4,
            fmt: "abc%%",
            want: "abc%",
            ..PLAIN
        },
        // case where escaped % does not fit
        Case {
            cells: 3,
            fmt: "abc%%abcabc",
            want: "<bc",
            ..PLAIN
        },
        // escaped % is first
        Case {
            cells: 1,
            fmt: "%%",
            want: "%",
            ..PLAIN
        },
        // ...and the format goes on afterwards. New here: every one of the
        // spec's three rows ended at the `%%` or ended in the same two
        // characters as the truncated answer, so a `%%` that swallowed the
        // byte after it changed no expectation.
        Case {
            cells: 4,
            fmt: "a%%b",
            want: "a%b",
            want_cells: Some(3),
            ..PLAIN
        },
    ]);
}

/// A format with far more items than the initial item table holds must
/// still render, and must render correctly rather than reporting an error.
///
/// The renderer starts with room for `STL_INITIAL_ITEMS` items and grows;
/// these are the spec's four ways of overrunning that — a thousand
/// highlight items, a hundred separators, a hundred stray characters, and
/// forty separators with text between them. The formats are built here
/// rather than written out, which is what the Lua did too.
#[test]
fn a_format_with_more_items_than_fit_still_renders() {
    /// `STL_INITIAL_ITEMS` in `statusline/stl`: the item table's first size.
    const INITIAL_ITEMS: usize = 20;

    let tabline: String = (1..=1000)
        .map(|i| {
            let group = if i % 2 == 0 {
                "%#TabLineSel#"
            } else {
                "%#TabLineFill#"
            };
            format!("{group}{}", i % 2)
        })
        .collect();

    let stl = Statusline::claim();
    for (fmt, want) in [
        (tabline, "<1010101010101010101"),
        ("%=".repeat(INITIAL_ITEMS * 5), "                    "),
        (
            format!("a{}", "a".repeat(INITIAL_ITEMS * 5)),
            "<aaaaaaaaaaaaaaaaaaa",
        ),
        (
            format!("a{}", "%=a".repeat(INITIAL_ITEMS * 2)),
            "a<aaaaaaaaaaaaaaaaaa",
        ),
    ] {
        // The formats are built at run time, so this row cannot go in a
        // `&'static str` table; the check is the same one.
        let case = Case {
            cells: 20,
            fmt: "",
            want: "",
            ..PLAIN
        };
        stl.check_owned(&case, &fmt, want, " ");
    }
}
