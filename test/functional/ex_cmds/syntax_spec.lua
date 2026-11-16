local t = require('test.testutil')
local n = require('test.functional.testnvim')()

local eq = t.eq
local pcall_err = t.pcall_err
local clear = n.clear
local command = n.command

describe(':syntax', function()
  before_each(clear)

  describe('keyword', function()
    it('does not crash when group name contains unprintable characters', function()
      eq(
        'Vim(syntax):E669: Unprintable character in group name',
        pcall_err(command, 'syntax keyword \024 foo bar')
      )
    end)
  end)
end)

describe(':syntax match', function()
  before_each(clear)

  it('with a negative leading context does not search past the line', function()
    -- lc=-1 starts the search one column after the current one, which on
    -- an empty line (or at its end) is past the NUL.
    command('syntax match X /foo/lc=-1')
    command('redraw')
    n.api.nvim_buf_set_lines(0, 0, -1, true, { 'afoo', '' })
    command('syntax match Y /o/lc=-1')
    command('redraw')
    command('set regexpengine=1')
    command('syntax match Z /foo/lc=-1')
    command('redraw')
    eq({ 'afoo', '' }, n.api.nvim_buf_get_lines(0, 0, -1, true))
    n.assert_alive()
  end)
end)
