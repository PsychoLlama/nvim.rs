local t = require('test.testutil')
local n = require('test.functional.testnvim')()

local Screen = require('test.functional.ui.screen')

local clear = n.clear
local command = n.command
local eq = t.eq
local feed = n.feed
local fn = n.fn
local pcall_err = t.pcall_err

describe('search (/)', function()
  before_each(clear)

  it('fails with huge column (%c) value #9930', function()
    eq([[Vim:E951: \% value too large]], pcall_err(command, '/\\v%18446744071562067968c'))
    eq([[Vim:E951: \% value too large]], pcall_err(command, '/\\v%2147483648c'))
  end)

  -- Upstream remembers the last leg of a `;` chain with one byte too many
  -- (its terminator), so `n` echoed the pattern and lost the count behind
  -- that NUL.
  it('remembers the last leg of a ; chain at its own length', function()
    local screen = Screen.new(40, 4)
    command('set shortmess-=S')
    fn.setline(1, { 'a b', 'b', 'b' })
    feed('/a/;/b<CR>')
    eq('b', fn.getreg('/'))
    feed('gg0n')
    screen:expect({ any = '/b%s+%[1/3%]' })
  end)

  -- Upstream searched for the :substitute pattern but remembered an empty
  -- search pattern, after which `n` matched everywhere.
  it('// with only a :substitute pattern remembers it', function()
    fn.setline(1, { 'x foo', 'foo', 'foo' })
    command('s/foo/bar/')
    feed('gg0//<CR>')
    eq({ 2, 0 }, { fn.line('.'), fn.col('.') - 1 })
    eq('foo', fn.getreg('/'))
    feed('n')
    eq({ 3, 0 }, { fn.line('.'), fn.col('.') - 1 })
  end)
end)
