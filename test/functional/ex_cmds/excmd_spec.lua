local t = require('test.testutil')
local n = require('test.functional.testnvim')()

local command = n.command
local eq = t.eq
local clear = n.clear
local fn = n.fn
local pcall_err = t.pcall_err
local assert_alive = n.assert_alive

describe('Ex cmds', function()
  before_each(function()
    clear()
  end)

  local function check_excmd_err(cmd, err)
    eq(err .. ': ' .. cmd, pcall_err(command, cmd))
  end

  it('handle integer overflow from user-input #5555', function()
    command(':9999999999999999999999999999999999999999')
    command(':later 9999999999999999999999999999999999999999')
    command(':echo expand("#<9999999999999999999999999999999999999999")')
    command(':lockvar 9999999999999999999999999999999999999999')
    command(
      ':winsize 9999999999999999999999999999999999999999 9999999999999999999999999999999999999999'
    )
    check_excmd_err(
      ':tabnext 9999999999999999999999999999999999999999',
      'Vim(tabnext):E475: Invalid argument: 9999999999999999999999999999999999999999'
    )
    eq(
      'Vim(Next):E163: There is only one file to edit',
      pcall_err(command, ':N 9999999999999999999999999999999999999999')
    )
    check_excmd_err(
      ':bdelete 9999999999999999999999999999999999999999',
      'Vim(bdelete):E516: No buffers were deleted'
    )
    eq(
      'Vim(menu):E329: No menu "9999999999999999999999999999999999999999"',
      pcall_err(command, ':menu 9999999999999999999999999999999999999999')
    )
    eq(
      'Vim(substitute):E1510: Value too large: 9999999999999999999999999999999999999999',
      pcall_err(command, ':%s/./b/9999999999999999999999999999999999999999')
    )
    assert_alive()
  end)

  it('listing long user command does not crash', function()
    command('execute "command" repeat("T", 255) ":"')
    command('command')
  end)

  it(':def is an unknown command #23149', function()
    eq('Vim:E492: Not an editor command: def', pcall_err(command, 'def'))
    eq(1, fn.exists(':d'))
    eq('delete', fn.fullcommand('d'))
    eq(1, fn.exists(':de'))
    eq('delete', fn.fullcommand('de'))
    eq(0, fn.exists(':def'))
    eq('', fn.fullcommand('def'))
    eq(1, fn.exists(':defe'))
    eq('defer', fn.fullcommand('defe'))
    eq(2, fn.exists(':defer'))
    eq('defer', fn.fullcommand('defer'))
  end)

  it('various command abbreviations', function()
    -- :connect needs at least :conn
    eq('change', fn.fullcommand('c'))
    eq('copy', fn.fullcommand('co'))
    eq('continue', fn.fullcommand('con'))
    eq('connect', fn.fullcommand('conn'))
    -- :restart needs at least :rest
    eq('read', fn.fullcommand('r'))
    eq('read', fn.fullcommand('re'))
    eq('resize', fn.fullcommand('res'))
    eq('restart', fn.fullcommand('rest'))
  end)
end)

-- A delimiter byte at or above 0x80 is handed to the pattern walk the way
-- upstream hands it: as a `char` (sign-extended) by :global, :sort, :catch,
-- :match and :syntax, so it ends the pattern there; as an unsigned byte by
-- :substitute, :filter and a /pat/ address, so it never does.  Expected
-- values were recorded on upstream Nvim 0.12.5.
describe('high-byte pattern delimiter', function()
  before_each(clear)

  local function run(cmd)
    n.exec([[
      enew!
      call setline(1, ['foo', 'bar', 'foo2'])
      call clearmatches()
      syntax clear
    ]])
    local ok, err = pcall(command, cmd)
    return n.api.nvim_buf_get_lines(0, 0, -1, true), (not ok) and err or nil
  end

  for _, d in ipairs({ '\xa7', '\xff' }) do
    local hex = string.format('0x%02X', d:byte())
    it(':global and :vglobal end the pattern at ' .. hex, function()
      eq({ { 'bar' } }, { run('g' .. d .. 'foo' .. d .. 'd') })
      eq({ { 'foo', 'foo2' } }, { run('v' .. d .. 'foo' .. d .. 'd') })
    end)

    it(':sort ends the pattern at ' .. hex, function()
      eq({ { 'bar', 'foo', 'foo2' } }, { run('sort ' .. d .. 'o' .. d) })
    end)

    it(':catch ends the pattern at ' .. hex, function()
      eq({ { 'caught', 'bar', 'foo2' } }, {
        run(
          'try | throw "foo" | catch ' .. d .. 'fo' .. d .. ' | call setline(1, "caught") | endtry'
        ),
      })
    end)

    it(':match ends the pattern at ' .. hex, function()
      run('match Search ' .. d .. 'foo' .. d)
      eq({ { group = 'Search', pattern = 'foo', priority = 10, id = 1 } }, fn.getmatches())
    end)

    it(':syntax ends the pattern at ' .. hex, function()
      eq({ { 'foo', 'bar', 'foo2' } }, { run('syn sync linecont ' .. d .. 'foo' .. d) })
      eq({ { 'foo', 'bar', 'foo2' } }, { run('syn match Foo ' .. d .. 'foo' .. d) })
      eq(1, fn.hlexists('Foo'))
    end)

    it(':substitute and a /pat/ address do not end the pattern at ' .. hex, function()
      local lines, err = run('s' .. d .. 'foo' .. d .. 'X' .. d)
      eq({ 'foo', 'bar', 'foo2' }, lines)
      t.matches('E486: Pattern not found: foo' .. d .. 'X' .. d, err)
      lines, err = run(d .. 'bar' .. d .. 'd')
      eq({ 'foo', 'bar', 'foo2' }, lines)
      t.matches('E492: Not an editor command', err)
    end)
  end

  it(':filter does not end the pattern at 0xA7', function()
    local _, err = run('filter \xa7bar\xa7 g/./p')
    t.matches('E476: Invalid command', err)
  end)
end)
