//! Where the completion popup goes and how big it is.
//!
//! The oracle the `popupmenu/` retype is checked against. `pum_display`
//! decides the menu's rows, columns, width and whether it scrolls from the
//! items' text widths, the cursor's place on the screen, 'pumheight',
//! 'pumwidth', 'pummaxwidth' and 'completeitemalign'; the answer is
//! published to `CompleteChanged` as `v:event` and to `pum_getpos()`. The
//! cases open the menu with `complete()` from inside Insert mode and record
//! what `CompleteChanged` saw.
//! Each answer is today's, and upstream 0.12.6's in an 80 x 24 screen.
//!
//! The unit editor's window starts 0 x 0; each case gives it the 80 x 24
//! screen's text area (80 x 22) for its own life.

#![cfg(not(miri))]

use crate::support::{Probe, Sandbox};
use neovim::window::{win_new_height, win_new_width};
use neovim::winlayer::Win;

struct Sized;

impl Sized {
    fn new(_fx: &Probe<'_>) -> Sized {
        win_new_width(Win::current(), 80);
        win_new_height(Win::current(), 22);
        Sized
    }
}

impl Drop for Sized {
    fn drop(&mut self) {
        win_new_width(Win::current(), 0);
        win_new_height(Win::current(), 0);
    }
}

/// `Open(items)`: complete `items` at the cursor and record the first
/// `CompleteChanged`'s `v:event` (less `completed_item`) in `g:pum`.
const PRELUDE: &str = r#"
func! Open(items)
  call complete(col('.'), a:items)
  return ''
endfunc
func! Record()
  if exists('g:pum') | return | endif
  let g:pum = filter(copy(v:event), {k -> k != 'completed_item'})
endfunc
augroup unit_pum
  autocmd!
  autocmd CompleteChanged * call Record()
augroup END
"#;

fn setup(sandbox: &Sandbox, fx: &Probe<'_>) {
    fx.home();
    sandbox.write("prelude.vim", PRELUDE.as_bytes());
    fx.run("source prelude.vim");
    fx.run(
        "set completeopt=menuone,noinsert pumheight=0 pumwidth=15 pummaxwidth=0 completeitemalign&",
    );
    // No "match 1 of 3" and no mode message: nothing here is about them.
    fx.run("set shortmess+=c noshowmode");
}

fn teardown(fx: &Probe<'_>) {
    // Showing the menu redraws the screen while the window is sized. Unless
    // a full redraw settles it again before the window goes back to 0 x 0,
    // the next case anywhere in the process that prints an error stops at a
    // hit-enter prompt (`printf_oracle` did, reading stdin forever).
    fx.run("redraw!");
    fx.run("silent! autocmd! unit_pum | silent! augroup! unit_pum");
    fx.run("silent! delfunction Open | silent! delfunction Record");
    fx.run("silent! unlet g:pum");
    fx.run("set completeopt& pumheight& pumwidth& pummaxwidth& completeitemalign& shortmess& showmode&");
    fx.home();
}

/// Put the cursor at (`line`, `col`) of a buffer of `lines` blank lines,
/// open the menu there with `items` (a Vimscript list), and answer the
/// recorded event as `[row, col, width, height, size, scrollbar]`.
fn open_at(fx: &Probe<'_>, line: u32, col: u32, items: &str) -> String {
    fx.run("silent! unlet g:pum");
    fx.run("silent! %delete _");
    fx.run(&format!(
        "call setline(1, repeat([repeat(' ', 79)], 22)) | call cursor({line}, {col})"
    ));
    fx.run(&format!(
        "silent execute \"normal! i\\<C-r>=Open({})\\<CR>\\<Esc>\"",
        items.replace('"', "\\\"")
    ));
    fx.eval("exists('g:pum') ? map(['row', 'col', 'width', 'height', 'size', 'scrollbar'], {_, k -> g:pum[k]}) : 'none'")
}

#[test]
fn the_menu_sits_below_the_cursor_and_fits_its_items() {
    let sandbox = Sandbox::dir("pum_layout");
    let fx = Probe::new(sandbox.editor());
    let _sized = Sized::new(&fx);
    setup(&sandbox, &fx);
    let words = "['alpha', 'beta', 'gamma']";
    let long = "['a-rather-long-completion-item-text', 'short']";
    let many = "map(range(40), {i -> 'item' . i})";
    let rows: &[(&str, String, &str)] = &[
        // Three short items: 'pumwidth' is the floor on the width.
        (
            "short",
            open_at(&fx, 1, 1, words),
            "[1.0, 0.0, 15.0, 3.0, 3, v:false]",
        ),
        // Wider than 'pumwidth': as wide as the widest item.
        (
            "long",
            open_at(&fx, 1, 1, long),
            "[1.0, 0.0, 35.0, 2.0, 2, v:false]",
        ),
        // More than fit below: the menu scrolls, and takes every row left.
        (
            "many",
            open_at(&fx, 1, 1, many),
            "[1.0, 0.0, 15.0, 22.0, 40, v:true]",
        ),
        // At the bottom of the window it goes above the cursor.
        (
            "above",
            open_at(&fx, 22, 1, words),
            "[18.0, 0.0, 15.0, 3.0, 3, v:false]",
        ),
        // Near the right edge it is pushed left to fit, one cell wider.
        (
            "right",
            open_at(&fx, 1, 75, words),
            "[1.0, 64.0, 16.0, 3.0, 3, v:false]",
        ),
    ];
    fx.run("set pumheight=5");
    let capped = open_at(&fx, 1, 1, many);
    fx.run("set pumheight=0 pummaxwidth=10");
    let narrow = open_at(&fx, 1, 1, long);
    fx.run("set pummaxwidth=0 pumwidth=1");
    // A kind and a menu column add to the width.
    let menu = open_at(
        &fx,
        1,
        1,
        "[{'word': 'alpha', 'kind': 'f', 'menu': '[mod]'}, {'word': 'beta', 'kind': 'v'}]",
    );
    fx.run("set completeitemalign=kind,abbr,menu");
    let realigned = open_at(
        &fx,
        1,
        1,
        "[{'word': 'alpha', 'kind': 'f', 'menu': '[mod]'}, {'word': 'beta', 'kind': 'v'}]",
    );
    teardown(&fx);
    let mut moved = Vec::new();
    for (name, got, want) in rows {
        if got != want {
            moved.push(format!("{name}: {got}"));
        }
    }
    for (name, got, want) in [
        // 'pumheight' caps the rows.
        ("capped", &capped, "[1.0, 0.0, 15.0, 5.0, 40, v:true]"),
        // 'pummaxwidth' caps the width.
        ("narrow", &narrow, "[1.0, 0.0, 10.0, 2.0, 2, v:false]"),
        // `alpha`, a space, `f`, a space, `[mod]`, and the padding cell.
        ("menu", &menu, "[1.0, 0.0, 14.0, 2.0, 2, v:false]"),
        // The order of the columns does not change the width.
        ("realigned", &realigned, "[1.0, 0.0, 14.0, 2.0, 2, v:false]"),
    ] {
        if got != want {
            moved.push(format!("{name}: {got}"));
        }
    }
    assert!(moved.is_empty(), "rows moved:\n{}", moved.join("\n"));
}
