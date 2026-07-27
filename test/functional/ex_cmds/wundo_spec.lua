-- Specs for :wundo and underlying functions

local t = require('test.testutil')
local n = require('test.functional.testnvim')()

local command = n.command
local clear = n.clear
local eval = n.eval
local fn = n.fn
local set_session = n.set_session
local eq = t.eq
local write_file = t.write_file

describe(':wundo', function()
  before_each(clear)
  after_each(function()
    os.remove(eval('getcwd()') .. '/foo')
  end)

  it('safely fails on new, non-empty buffer', function()
    command('normal! iabc')
    command('wundo foo') -- This should not segfault. #1027
    --TODO: check messages for error message
  end)
end)

describe('u_* functions', function()
  it('safely fail on new, non-empty buffer', function()
    local session = n.new_session(false, {
      args = {
        '-c',
        'set undodir=. undofile',
      },
    })
    set_session(session)
    command('echo "True"') -- Should not error out due to crashed Neovim
    session:close()
  end)
end)

describe('the undo file format', function()
  local tmpdir

  before_each(function()
    clear()
    tmpdir = t.tmpname(false)
    t.mkdir(tmpdir)
    command('set undodir=' .. tmpdir .. ' undofile')
  end)

  after_each(function()
    n.rmdir(tmpdir)
  end)

  -- The undo file is on-disk state that outlives the editor: an nvim that
  -- writes a header any other build cannot read silently loses every user's
  -- persistent undo. These bytes are the contract.
  it('starts with its magic and version', function()
    local target = tmpdir .. '/Xundo_format'
    write_file(target, 'alpha\nbravo\n')
    command('edit ' .. fn.fnameescape(target))
    command('normal! Gdd')
    command('write')

    local raw = t.read_file(fn.undofile(target))

    -- "Vim\x9fUnDo\xe5", then the two-byte version 3, then a 32-byte hash.
    eq('Vim\159UnDo\229', raw:sub(1, 9))
    eq('\0\3', raw:sub(10, 11))
    assert(#raw > 11 + 32, 'the hash and the tree should follow the header')
  end)

  it('round-trips a branching history', function()
    local target = tmpdir .. '/Xundo_branch'
    write_file(target, 'alpha\nbravo\ncharlie\n')
    command('edit ' .. fn.fnameescape(target))
    command('normal! Gdd')
    command('let &undolevels = &undolevels')
    command('normal! ggIfirst ')
    command('let &undolevels = &undolevels')
    command('undo')
    command('normal! Azzz')
    command('write')

    local states = {}
    for seq = 0, 4 do
      command('silent! undo ' .. seq)
      states[#states + 1] = table.concat(fn.getline(1, '$'), '|')
    end

    -- Drop the tree and read it back off disk; every sequence number must
    -- land on the same buffer contents, including the abandoned branch.
    command('bwipe!')
    command('edit ' .. fn.fnameescape(target))
    command('rundo ' .. fn.fnameescape(fn.undofile(target)))
    local reread = {}
    for seq = 0, 4 do
      command('silent! undo ' .. seq)
      reread[#reread + 1] = table.concat(fn.getline(1, '$'), '|')
    end
    eq(states, reread)
  end)
end)
