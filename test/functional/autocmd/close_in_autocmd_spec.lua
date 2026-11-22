-- Closing windows and tab pages from user code the editor runs in the middle
-- of its own work. Each case once left Rust holding a window across the
-- handler, or looping for good.

local t = require('test.testutil')
local n = require('test.functional.testnvim')()

local clear, eq = n.clear, t.eq
local command, eval, exec = n.command, n.eval, n.exec
local api = n.api
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

describe(':tabclose', function()
  before_each(clear)

  it('stops at a floating window that refuses to close', function()
    command('set nohidden')
    command('tabnew')
    local buf = api.nvim_create_buf(true, false)
    api.nvim_open_win(buf, true, { relative = 'editor', row = 1, col = 1, width = 5, height = 2 })
    api.nvim_buf_set_lines(buf, 0, -1, true, { 'changed' })
    -- The float's buffer is changed and there is no `!`: it stays, and so
    -- does the tab page. This used to retry the float forever.
    t.matches('E37', t.pcall_err(command, 'tabclose'))
    assert_alive()
    eq(2, eval('tabpagenr("$")'))
    eq(true, api.nvim_win_get_config(0).relative ~= '')
    command('tabclose!')
    eq(1, eval('tabpagenr("$")'))
  end)
end)
