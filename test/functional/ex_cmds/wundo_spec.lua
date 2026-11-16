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

  -- Where the file's first undo header record starts (the offset, 1-based,
  -- of its first field after the magic), and where each of its entry
  -- records starts (the offset of its `top` field). Walks the fixed layout
  -- the writer lays down rather than searching for the magic, which may
  -- turn up inside the hash.
  local function first_header(raw)
    local at = 1
    local function u32()
      local b1, b2, b3, b4 = raw:byte(at, at + 3)
      at = at + 4
      return ((b1 * 256 + b2) * 256 + b3) * 256 + b4
    end
    local function trailer()
      while raw:byte(at) ~= 0 do
        at = at + 2 + raw:byte(at)
      end
      at = at + 1
    end
    at = 9 + 2 + 32 + 4 + 1
    at = at + u32() -- the U line
    at = at + 4 + 4 + 6 * 4 + 8
    trailer()
    assert(raw:sub(at, at + 1) == '\95\208', 'a header record follows the file header')
    local header = at + 2
    at = at + 2 + 5 * 4 + 12 + 4 + 2 + 26 * 12 + 32 + 8
    trailer()
    local entries = {}
    while raw:sub(at, at + 1) == '\245\024' do
      at = at + 2
      entries[#entries + 1] = at
      at = at + 12
      for _ = 1, u32() do
        at = at + u32()
      end
    end
    assert(#entries > 0, 'the header carries an entry')
    return header, entries
  end

  -- An entry covers the lines between its `top` and `bot`. A file whose
  -- entry names a line below the first one is refused when it is read; it
  -- used to pass the apply-time check against the buffer's length and walk
  -- the memline out of range on the next undo.
  it('refuses an entry whose line numbers no buffer has', function()
    local target = tmpdir .. '/Xundo_lines'
    write_file(target, 'alpha\nbeta\ngamma\nlast\n')
    command('edit ' .. fn.fnameescape(target))
    command('normal! jdd')
    command('write')
    -- Read only by hand from here, so that each refusal is the one asked for.
    command('set noundofile')
    local undofile = fn.undofile(target)
    local raw = t.read_file(undofile)
    local _, entries = first_header(raw)
    local top = entries[1]
    for _, bad in ipairs({
      '\255\000\000\001', -- a negative top
      '\000\000\000\003\000\000\000\002', -- bot above top
    }) do
      write_file(undofile, raw:sub(1, top - 1) .. bad .. raw:sub(top + #bad), true)
      command('bwipe!')
      command('edit ' .. fn.fnameescape(target))
      t.matches(
        'E825: Corrupted undo file %(entry line numbers%)',
        t.pcall_err(command, 'rundo ' .. fn.fnameescape(undofile))
      )
      command('silent! undo 0')
      eq({ 'alpha', 'gamma', 'last' }, fn.getline(1, '$'))
    end
  end)

  -- The header's numbers are the file's word: a sequence or save number
  -- the buffer could not step past is refused, and a cursor line or time
  -- at the end of its range is used without overflowing.
  it('reads header numbers at the ends of their range', function()
    local target = tmpdir .. '/Xundo_numbers'
    write_file(target, 'alpha\nbeta\ngamma\n')
    command('edit ' .. fn.fnameescape(target))
    command('normal! jdd')
    command('write')
    command('set noundofile')
    local undofile = fn.undofile(target)
    local raw = t.read_file(undofile)
    local header = first_header(raw)
    -- Offsets into the header record: the links and sequence number, the
    -- cursor, its column, the flags, the marks, the Visual area, the time,
    -- then the save number's optional field.
    local seq, cursor, time = header + 16, header + 20, header + 20 + 12 + 4 + 2 + 26 * 12 + 32
    local save_nr = time + 8 + 2
    eq('\004\001', raw:sub(time + 8, time + 9), 'the save number follows the time')
    local function patched(at, bytes)
      write_file(undofile, raw:sub(1, at - 1) .. bytes .. raw:sub(at + #bytes), true)
      command('bwipe!')
      command('edit ' .. fn.fnameescape(target))
      return 'rundo ' .. fn.fnameescape(undofile)
    end
    t.matches(
      'E825: Corrupted undo file %(uh_seq%)',
      t.pcall_err(command, patched(seq, '\127\255\255\255'))
    )
    t.matches(
      'E825: Corrupted undo file %(counter%)',
      t.pcall_err(command, patched(save_nr, '\128\000\000\000'))
    )
    for _, case in ipairs({
      { cursor, '\127\255\255\255' }, -- the line the cursor goes back to
      { time, '\128\000\000\000\000\000\000\000' }, -- when the change was
    }) do
      command(patched(case[1], case[2]))
      command('undo 0')
      eq({ 'alpha', 'beta', 'gamma' }, fn.getline(1, '$'))
    end
  end)
end)
