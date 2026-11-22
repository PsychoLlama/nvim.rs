-- Closing windows and tab pages from user code the editor runs in the middle
-- of its own work. Each case once left Rust holding a window across the
-- handler.

local t = require('test.testutil')
local n = require('test.functional.testnvim')()

local clear, eq = n.clear, t.eq
local command, eval, exec = n.command, n.eval, n.exec
local assert_alive = n.assert_alive

describe('closing windows from an autocommand', function()
  before_each(clear)

  it('is refused in DiffUpdated, which fires while a window is being diffed', function()
    exec([[
      call setline(1, ['a', 'b', 'c'])
      diffthis
      vnew
      call setline(1, ['a', 'x', 'c'])
      let g:errs = []
      autocmd DiffUpdated * call add(g:errs, luaeval('select(2, pcall(vim.api.nvim_win_close, 0, true))'))
    ]])
    -- Setting 'diff' brings the diff up to date from inside the option
    -- setter, which holds this window.
    command('diffthis')
    assert_alive()
    eq(2, eval('winnr("$")'))
    t.matches('E1312', eval('g:errs[0]'))

    -- A redraw's filler count recomputes a stale diff as well.
    command('autocmd! DiffUpdated')
    command('autocmd DiffUpdated * ++once silent! only!')
    command('call setline(2, "y") | redraw!')
    assert_alive()
    eq(2, eval('winnr("$")'))
  end)

  it('is refused in DiffUpdated when the handler wipes a diffed buffer', function()
    exec([[
      call setline(1, ['a', 'b', 'c'])
      diffthis
      vnew
      call setline(1, ['a', 'x', 'c'])
      let g:other = bufnr('#')
      autocmd DiffUpdated * ++once silent! exe 'bwipe!' g:other
    ]])
    command('diffthis')
    assert_alive()
    eq(2, eval('winnr("$")'))
  end)
end)
