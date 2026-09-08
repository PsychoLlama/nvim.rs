-- Timing canary for the spell subsystem (p26-11).
--
--   nvim --headless -c 'luafile spellbench.lua' -c 'qa!'
--
-- Run it through spellbench.sh, which does the interleaved A/B the drift
-- rule requires. Prints "phase<TAB>ms" lines prefixed with SPELLBENCH.
--
-- There is no spell case in test/benchmark, and 'spell' is redrawn on every
-- keystroke, so this is the canary the slice that rewrote the word tries
-- owes. Phases:
--
--   ctl        nothing            -- noise floor (strdisplaywidth)
--   check      find_word          -- spellbadword() over a 5000-line buffer:
--                                     one tree descent per word, which is
--                                     what 'spell' costs while typing
--   redraw     spell_check + draw -- the real path: 'spell' on, scrolled
--                                     through the same buffer with redraws
--   suggest    suggest_trie_walk  -- z=/spellsuggest() over 40 misspellings:
--                                     the edit walk, REP items and the
--                                     sound-a-like pass
--   soundfold  spell_soundfold    -- the SAL rule engine on its own
--
-- The word list is fixed so both binaries see the same tree descents; the
-- language is the shipped en.utf-8.spl, which is git-tracked.

-- stylua: ignore
local WORDS = {
  'the', 'quick', 'brown', 'fox', 'jumps', 'over', 'lazy', 'dog', 'and',
  'then', 'runs', 'through', 'forest', 'where', 'birds', 'sing', 'their',
  'morning', 'songs', 'while', 'river', 'flows', 'quietly', 'beneath',
  'ancient', 'bridge', 'children', 'played', 'summer', 'evening', 'until',
  'stars', 'appeared', 'above', 'mountain', 'ridge', 'farmer', 'walked',
  'along', 'narrow', 'path', 'carrying', 'basket', 'filled', 'apples',
  'pears', 'plums', 'gathered', 'orchard', 'behind', 'stone', 'cottage',
  'smoke', 'curled', 'chimney', 'wind', 'carried', 'scent', 'woodsmoke',
  'across', 'valley', 'toward', 'distant', 'village', 'bells', 'rang',
  'calling', 'people', 'gather', 'square', 'market', 'traders', 'shouted',
  'prices', 'wool', 'grain', 'salt', 'iron', 'travellers', 'rested',
  'benches', 'drinking', 'water', 'well', 'centre', 'watching', 'clouds',
  'gather', 'promise', 'rain', 'before', 'nightfall', 'settled', 'over',
  'roofs', 'lanterns', 'lit', 'windows', 'one', 'after', 'another',
}

-- stylua: ignore
local BAD = {
  'teh', 'quik', 'browm', 'jmups', 'ovre', 'lasy', 'thrugh', 'forrest',
  'birdz', 'singg', 'thier', 'mornning', 'rivver', 'quietley', 'benethe',
  'anceint', 'bridg', 'childrenn', 'plaied', 'sumer', 'evning', 'untill',
  'starz', 'apeared', 'mountian', 'ridg', 'farmar', 'walkd', 'narow',
  'carying', 'baskett', 'appels', 'gatherd', 'orchad', 'behinde',
  'cottag', 'chimny', 'valey', 'vilage', 'lanterms',
}

local function build(lines)
  local out = {}
  local w = 1
  for _ = 1, lines do
    local parts = {}
    for _ = 1, 10 do
      parts[#parts + 1] = WORDS[w]
      w = w % #WORDS + 1
    end
    out[#out + 1] = table.concat(parts, ' ')
  end
  return out
end

local function ms(f)
  local t = vim.uv.hrtime()
  f()
  return (vim.uv.hrtime() - t) / 1e6
end

local function say(name, value)
  io.stdout:write(('SPELLBENCH\t%s\t%.3f\n'):format(name, value))
end

local text = build(5000)
vim.api.nvim_buf_set_lines(0, 0, -1, false, text)
vim.o.spelllang = 'en'

-- A round that could not load the language, or that was interrupted, would
-- otherwise contribute a spuriously small minimum for every phase after it.
-- Print nothing at all rather than a number that means something else.
if vim.fn.spellbadword('teh')[2] ~= 'bad' or #vim.fn.spellsuggest('teh', 3) == 0 then
  io.stderr:write('spellbench: en.utf-8.spl did not load\n')
  os.exit(1)
end

-- ctl: the noise floor, nothing to do with spelling.
say(
  'ctl',
  ms(function()
    local n = 0
    for _ = 1, 20 do
      for i = 1, #text do
        n = n + vim.fn.strdisplaywidth(text[i])
      end
    end
  end)
)

-- check: one tree descent per word.
say(
  'check',
  ms(function()
    for i = 1, #text do
      vim.fn.spellbadword(text[i])
    end
  end)
)

-- redraw: 'spell' on, scrolled through the buffer.
vim.o.spell = true
say(
  'redraw',
  ms(function()
    for i = 1, 5000, 20 do
      vim.api.nvim_win_set_cursor(0, { i, 0 })
      vim.cmd('redraw')
    end
  end)
)
vim.o.spell = false

-- suggest: the edit walk plus the sound-a-like pass.
vim.o.spellsuggest = 'fast'
say(
  'sug_fast',
  ms(function()
    for _, w in ipairs(BAD) do
      vim.fn.spellsuggest(w, 5)
    end
  end)
)
vim.o.spellsuggest = 'best'
say(
  'suggest',
  ms(function()
    for _, w in ipairs(BAD) do
      vim.fn.spellsuggest(w, 5)
    end
  end)
)

-- soundfold: the SAL engine on its own.
say(
  'soundfold',
  ms(function()
    for _ = 1, 200 do
      for _, w in ipairs(WORDS) do
        vim.fn.soundfold(w)
      end
    end
  end)
)
