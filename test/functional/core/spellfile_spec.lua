local t = require('test.testutil')
local n = require('test.functional.testnvim')()

local eq = t.eq
local pcall_err = t.pcall_err
local clear = n.clear
local api = n.api
local fn = n.fn
local rmdir = n.rmdir
local write_file = t.write_file
local mkdir = t.mkdir

local testdir = 'Xtest-functional-spell-spellfile.d'

describe('spellfile', function()
  before_each(function()
    clear({ env = { XDG_DATA_HOME = testdir .. '/xdg_data' } })
    rmdir(testdir)
    mkdir(testdir)
    mkdir(testdir .. '/spell')
  end)
  after_each(function()
    rmdir(testdir)
  end)
  --                   ┌ Magic string (#VIMSPELLMAGIC)
  --                   │       ┌ Spell file version (#VIMSPELLVERSION)
  local spellheader = 'VIMspell\050'
  it('errors out when prefcond section is truncated', function()
    api.nvim_set_option_value('runtimepath', testdir, {})
    -- stylua: ignore
    write_file(testdir .. '/spell/en.ascii.spl',
    --                         ┌ Section identifier (#SN_PREFCOND)
    --                         │   ┌ Section flags (#SNF_REQUIRED or zero)
    --                         │   │   ┌ Section length (4 bytes, MSB first)
               spellheader .. '\003\001\000\000\000\003'
    --             ┌ Number of regexes in section (2 bytes, MSB first)
    --             │       ┌ Condition length (1 byte)
    --             │       │   ┌ Condition regex (missing!)
               .. '\000\001\001')
    api.nvim_set_option_value('spelllang', 'en', {})
    t.matches('Vim%(set%):E758: Truncated spell file', pcall_err(n.command, 'set spell'))
  end)
  it('errors out when prefcond regexp contains NUL byte', function()
    api.nvim_set_option_value('runtimepath', testdir, {})
    -- stylua: ignore
    write_file(testdir .. '/spell/en.ascii.spl',
    --                         ┌ Section identifier (#SN_PREFCOND)
    --                         │   ┌ Section flags (#SNF_REQUIRED or zero)
    --                         │   │   ┌ Section length (4 bytes, MSB first)
               spellheader .. '\003\001\000\000\000\008'
    --             ┌ Number of regexes in section (2 bytes, MSB first)
    --             │       ┌ Condition length (1 byte)
    --             │       │   ┌ Condition regex
    --             │       │   │       ┌ End of sections marker
               .. '\000\001\005ab\000cd\255'
    --             ┌ LWORDTREE tree length (4 bytes)
    --             │               ┌ KWORDTREE tree length (4 bytes)
    --             │               │               ┌ PREFIXTREE tree length
               .. '\000\000\000\000\000\000\000\000\000\000\000\000')
    api.nvim_set_option_value('spelllang', 'en', {})
    t.matches('Vim%(set%):E759: Format error in spell file', pcall_err(n.command, 'set spell'))
  end)
  it('errors out when region contains NUL byte', function()
    api.nvim_set_option_value('runtimepath', testdir, {})
    -- stylua: ignore
    write_file(testdir .. '/spell/en.ascii.spl',
    --                         ┌ Section identifier (#SN_REGION)
    --                         │   ┌ Section flags (#SNF_REQUIRED or zero)
    --                         │   │   ┌ Section length (4 bytes, MSB first)
               spellheader .. '\000\001\000\000\000\008'
    --             ┌ Regions  ┌ End of sections marker
               .. '01234\00067\255'
    --             ┌ LWORDTREE tree length (4 bytes)
    --             │               ┌ KWORDTREE tree length (4 bytes)
    --             │               │               ┌ PREFIXTREE tree length
               .. '\000\000\000\000\000\000\000\000\000\000\000\000')
    api.nvim_set_option_value('spelllang', 'en', {})
    t.matches('Vim%(set%):E759: Format error in spell file', pcall_err(n.command, 'set spell'))
  end)
  it('errors out when SAL section contains NUL byte', function()
    api.nvim_set_option_value('runtimepath', testdir, {})
    -- stylua: ignore
    write_file(testdir .. '/spell/en.ascii.spl',
    --                         ┌ Section identifier (#SN_SAL)
    --                         │   ┌ Section flags (#SNF_REQUIRED or zero)
    --                         │   │   ┌ Section length (4 bytes, MSB first)
               spellheader .. '\005\001\000\000\000\008'
    --             ┌ salflags
    --             │   ┌ salcount (2 bytes, MSB first)
    --             │   │       ┌ salfromlen (1 byte)
    --             │   │       │   ┌ Special character
    --             │   │       │   │┌ salfrom (should not contain NUL)
    --             │   │       │   ││   ┌ saltolen
    --             │   │       │   ││   │   ┌ salto
    --             │   │       │   ││   │   │┌ End of sections marker
               .. '\000\000\001\0024\000\0017\255'
    --             ┌ LWORDTREE tree length (4 bytes)
    --             │               ┌ KWORDTREE tree length (4 bytes)
    --             │               │               ┌ PREFIXTREE tree length
               .. '\000\000\000\000\000\000\000\000\000\000\000\000')
    api.nvim_set_option_value('spelllang', 'en', {})
    t.matches('Vim%(set%):E759: Format error in spell file', pcall_err(n.command, 'set spell'))
  end)
  it('errors out when spell header contains NUL bytes', function()
    api.nvim_set_option_value('runtimepath', testdir, {})
    write_file(testdir .. '/spell/en.ascii.spl', spellheader:sub(1, -3) .. '\000\000')
    api.nvim_set_option_value('spelllang', 'en', {})
    t.matches(
      'Vim%(set%):E757: This does not look like a spell file',
      pcall_err(n.command, 'set spell')
    )
  end)

  it('can be set to a relative path', function()
    local fname = testdir .. '/spell/spell.add'
    api.nvim_set_option_value('spellfile', fname, {})
  end)

  describe('a shared sub-tree', function()
    local function be3(v)
      return string.char(math.floor(v / 65536) % 256, math.floor(v / 256) % 256, v % 256)
    end
    local function be4(v)
      return '\000' .. be3(v)
    end
    -- A case-folded tree whose root has 'a' and 'b'. 'a' leads down a chain
    -- of `depth` nodes on 'c' ending in a word; 'b' leads to one node whose
    -- 'c' names, by index, the chain node `ref` (an index into the tree)
    -- instead of a sub-tree of its own. `after` is the byte the reference
    -- hangs under.
    local function load(depth, ref, after)
      local nodes = { '\002ab' }
      local starts = {}
      local at = 3
      for _ = 1, depth do
        starts[#starts + 1] = at
        nodes[#nodes + 1] = '\001c'
        at = at + 2
      end
      nodes[#nodes + 1] = '\001\000'
      at = at + 2
      nodes[#nodes + 1] = '\001\001' .. be3(ref(starts)) .. (after or 'c')
      at = at + 2
      write_file(
        testdir .. '/spell/en.utf-8.spl',
        spellheader .. '\255' .. be4(at) .. table.concat(nodes) .. be4(0) .. be4(0),
        true
      )
      api.nvim_set_option_value('runtimepath', testdir, {})
      api.nvim_set_option_value('spelllang', 'en', {})
    end
    local function refused(...)
      load(...)
      return pcall_err(n.command, 'set spell')
    end
    local function loads(...)
      load(...)
      n.command('set spell')
    end

    it('is read where a finished node starts', function()
      -- 'b' shares the chain's second node: "acc" and "bcc" are words.
      loads(2, function(s)
        return s[2]
      end)
      eq({ '', '' }, fn.spellbadword('acc bcc'))
      eq('bc', fn.spellbadword('bc')[1])
    end)

    -- Each of these used to load, and the first walk over it indexed past
    -- the tree (a panic) or round a cycle.
    it('is refused anywhere else', function()
      local format = 'Vim%(set%):E759: Format error in spell file'
      -- Into the middle of a node: its byte 'c' read as a child count.
      t.matches(
        format,
        refused(2, function(s)
          return s[1] + 1
        end)
      )
      n.command('set nospell')
      -- The root, still being read: a cycle.
      t.matches(
        format,
        refused(2, function()
          return 0
        end)
      )
      n.command('set nospell')
      -- Under a NUL, which would make it a word end.
      t.matches(
        format,
        refused(2, function(s)
          return s[2]
        end, '\000')
      )
      n.command('set nospell')
      -- Hung one level deeper than it was read, the word ending under it
      -- would pass the 254 letters a word may have; one node down it fits.
      t.matches(
        format,
        refused(253, function(s)
          return s[1]
        end)
      )
      n.command('set nospell')
      loads(253, function(s)
        return s[2]
      end)
    end)
  end)

  it('can be set to an absolute path', function()
    local fname = fn.fnamemodify(testdir .. '/spell/spell.add', ':p')
    api.nvim_set_option_value('spellfile', fname, {})
  end)

  describe('default location', function()
    it("is stdpath('data')/site/spell/en.utf-8.add", function()
      n.command('set spell')
      n.insert('abc')
      n.feed('zg')
      eq(
        t.fix_slashes(fn.stdpath('data') .. '/site/spell/en.utf-8.add'),
        t.fix_slashes(api.nvim_get_option_value('spellfile', {}))
      )
    end)

    it("is not set if stdpath('data') is not writable", function()
      n.command('set spell')
      fn.writefile({ '' }, testdir .. '/xdg_data')
      n.insert('abc')
      eq("Vim(normal):E764: Option 'spellfile' is not set", pcall_err(n.command, 'normal! zg'))
    end)

    it("is not set if 'spelllang' is not set", function()
      n.command('set spell spelllang=')
      n.insert('abc')
      eq("Vim(normal):E764: Option 'spellfile' is not set", pcall_err(n.command, 'normal! zg'))
    end)
  end)
end)
