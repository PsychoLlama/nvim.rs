-- Timing canary for extmarks and the marktree under them.
--
--   nvim --headless -c 'set columns=200 lines=60' -c 'luafile markbench.lua' -c 'qa!'
--
-- Run it through markbench.sh, which does the interleaved A/B the drift rule
-- requires. Prints "phase<TAB>ms" lines prefixed with MARKBENCH so the shell
-- can pick them out of whatever else lands on stdout. `-c` rather than `-l`
-- for the same reason as scrbench: only `-c` sets full_screen, and the
-- `decor` phase needs a real 200x60 grid to draw into.
--
-- The fixture, built before any phase and always: a 20,000-line buffer
-- holding 40,000 extmarks in one namespace, placed by a fixed LCG -- one in
-- three a range (`end_row`/`end_col`, a paired mark), one in four with
-- `right_gravity = false`, one in five with a highlight, one in fifty with
-- end-of-line virtual text. Every phase reads that fixture, and every phase
-- but `del` and `clear` leaves it as it found it (the splice and undo phases
-- put back what they change), so a phase run alone reads the same tree it
-- would read in a full run.
--
-- Phase map (which code each one is the canary for):
--   ctl      nothing            -- noise floor: strwidth(), no marktree
--   set      marktree_put /     -- 40,000 marks with the fixture's mix put into
--            extmark_set           a scratch buffer, which is then wiped; the
--                                  wipe's marktree_clear is part of the phase
--   get      marktree_lookup    -- nvim_buf_get_extmark_by_id over random ids:
--                                  the id map and the walk to the key's node
--   iter     marktree_itr_get / -- nvim_buf_get_extmarks over 40-line windows:
--            marktree_itr_next     seek and walk in position order
--   overlap  marktree_itr_get_  -- the same windows with `overlap = true`: the
--            overlap / intersect   ranges that start above the window (the
--                                  intersection metadata, `meta.rs`)
--   splice   extmark_splice /   -- inserted text and lines, each removed again:
--            marktree_splice       every mark after the edit moves, ranges
--                                  straddling it stretch and shrink
--   undo     extmark/undo.rs /  -- a five-line delete that takes marks with it,
--            u_extmark_copy        then `:undo`, which restores them
--   decor    decor_redraw_* /   -- `redraw!` of a window scrolled over the
--            marktree_itr_*        fixture: highlights and virtual text per
--                                  line, the decoration state machine
--   del      marktree_del_itr / -- 20,000 marks deleted by id: node merges
--            rebalance             and the id map
--   clear    marktree_clear_ns  -- nvim_buf_clear_namespace over what is left
--            / marktree_del_itr
--
-- MARKBENCH_ONLY runs a comma-separated subset of the phases; the fixture is
-- always built, so `MARKBENCH_ONLY=none` is the setup to subtract from a
-- one-phase instruction count. `del` and `clear` shrink the fixture, so a
-- phase after them in a full run reads less than it does alone; read phases
-- one at a time.
--
-- Env: MARKBENCH_SCALE multiplies every phase's round count (default 1).
--
-- Noise, measured: three `--cachegrind` runs of one codegen-units = 1
-- release binary spread 0.016 % on the whole (5,077 M Ir). Each phase run
-- alone (setup included) spread at most 0.26 % (`none` 380 M, `undo`), the
-- rest 0.04-0.12 %; with the setup subtracted, `decor` 0.07 %, `set`
-- 0.12 %, `overlap`/`iter`/`splice`/`get` 0.23-0.33 %, `ctl` 0.5 %, `undo`
-- 0.9 %, and `del`/`clear` (70-78 M) 1.4-1.8 %. The residue is ~1 M Ir a
-- run whatever the phase, so read the small phases' differences as noise
-- under a couple of percent.

local api = vim.api
local SCALE = tonumber(os.getenv('MARKBENCH_SCALE') or '1')
local NLINE = 20000
local NMARK = 40000

local ONLY = os.getenv('MARKBENCH_ONLY')

-- Whether phase `name` is in MARKBENCH_ONLY's comma-separated list (every
-- phase when it is unset).
local function wanted(name)
  if not ONLY then
    return true
  end
  for p in ONLY:gmatch('[^,]+') do
    if p == name then
      return true
    end
  end
  return false
end

-- The collector runs only between phases. Left to itself it steps when the
-- heap crosses a threshold, and where that falls moves with the addresses
-- the process was given: a setup-only run spread 0.9 % between runs with it
-- on and 0.03 % with it off. A phase's own garbage is collected after it,
-- outside the timing but inside its instruction count.
collectgarbage('stop')

local out = {}
local function phase(name, f)
  if not wanted(name) then
    return
  end
  local t0 = vim.uv.hrtime()
  f()
  out[#out + 1] = string.format('MARKBENCH\t%s\t%.1f', name, (vim.uv.hrtime() - t0) / 1e6)
  collectgarbage('collect')
end

-- Cheap reproducible PRNG; Lua's math.random is not stable across builds.
-- Each user re-seeds it, so a phase draws the same numbers whether or not
-- the phases before it ran.
local seed
local function reseed(s)
  seed = s
end
local function rnd(n)
  seed = (seed * 1103515245 + 12345) % 2147483648
  return seed % n + 1
end

-- Lines of varying length, so a mark's column is sometimes past the end of
-- a short line's neighbours and the walk is not one uniform shape.
local corpus = {}
for i = 1, NLINE do
  local n = i % 5
  if n == 0 then
    corpus[i] = string.format('local x%d = %d', i, i)
  elseif n == 1 then
    corpus[i] = string.rep('word ', 16) .. i
  elseif n == 2 then
    corpus[i] = '    if value_' .. i .. ' then return nil end'
  elseif n == 3 then
    corpus[i] = string.rep('ab', 40)
  else
    corpus[i] = '-- a comment line with some text in it ' .. i
  end
end

local hl = {
  api.nvim_get_hl_id_by_name('Comment'),
  api.nvim_get_hl_id_by_name('Keyword'),
  api.nvim_get_hl_id_by_name('String'),
  api.nvim_get_hl_id_by_name('Function'),
}

-- The fixture's k-th mark: its position and options, from the PRNG.
local function mark_spec(k)
  local row = rnd(NLINE) - 1
  local col = rnd(40) - 1
  local opts = {}
  if k % 3 == 0 then
    local end_row = row + rnd(4) - 1
    if end_row >= NLINE then
      end_row = NLINE - 1
    end
    opts.end_row = end_row
    opts.end_col = rnd(40) - 1
    if end_row == row and opts.end_col < col then
      opts.end_col = col
    end
  end
  if k % 4 == 0 then
    opts.right_gravity = false
    if opts.end_row then
      opts.end_right_gravity = true
    end
  end
  if k % 5 == 0 then
    opts.hl_group = hl[k % 4 + 1]
  end
  if k % 50 == 0 then
    opts.virt_text = { { 'virt ' .. k, 'Comment' } }
  end
  return row, col, opts
end

local function place(buf, ns)
  reseed(4242)
  local ids = {}
  for k = 1, NMARK do
    local row, col, opts = mark_spec(k)
    opts.strict = false
    ids[k] = api.nvim_buf_set_extmark(buf, ns, row, col, opts)
  end
  return ids
end

vim.o.swapfile = false
vim.o.undofile = false
vim.o.shada = ''
vim.o.laststatus = 2
vim.o.lazyredraw = false
vim.o.wrap = false

-- The fixture: buffer 1, every phase's tree.
local ns = api.nvim_create_namespace('markbench')
api.nvim_buf_set_lines(0, 0, -1, true, corpus)
local buf = api.nvim_get_current_buf()
vim.bo[buf].undolevels = -1
local ids = place(buf, ns)

-- Noise floor: a Vimscript call that reaches nothing extmark-shaped.
phase('ctl', function()
  for _ = 1, 100000 * SCALE do
    local _ = vim.fn.strwidth('word001 word002 word003 word004 word005')
  end
end)

phase('set', function()
  for _ = 1, SCALE do
    local scratch = api.nvim_create_buf(false, true)
    api.nvim_buf_set_lines(scratch, 0, -1, true, corpus)
    local placed = place(scratch, ns)
    assert(#placed == NMARK)
    api.nvim_buf_delete(scratch, { force = true })
  end
end)

phase('get', function()
  reseed(17)
  local total = 0
  for _ = 1, 200000 * SCALE do
    local pos = api.nvim_buf_get_extmark_by_id(buf, ns, ids[rnd(NMARK)], {})
    total = total + pos[1]
  end
  assert(total > 0)
end)

phase('iter', function()
  reseed(23)
  local seen = 0
  for _ = 1, 4000 * SCALE do
    local row = rnd(NLINE - 40) - 1
    seen = seen + #api.nvim_buf_get_extmarks(buf, ns, { row, 0 }, { row + 40, -1 }, {})
  end
  assert(seen > 0)
end)

phase('overlap', function()
  reseed(23)
  local seen = 0
  for _ = 1, 4000 * SCALE do
    local row = rnd(NLINE - 40) - 1
    seen = seen
      + #api.nvim_buf_get_extmarks(buf, ns, { row, 0 }, { row + 40, -1 }, { overlap = true })
  end
  assert(seen > 0)
end)

-- Each round puts text in and takes it out again, so the fixture comes back
-- whole: a mark at the insertion point moves right and back (or stays, by
-- its gravity), and a range across it stretches and shrinks.
phase('splice', function()
  reseed(31)
  local before = api.nvim_buf_get_extmark_by_id(buf, ns, ids[NMARK], {})
  for _ = 1, 6000 * SCALE do
    local row = rnd(NLINE - 1) - 1
    -- The shortest corpus line is 12 bytes.
    local col = rnd(10) - 1
    api.nvim_buf_set_text(buf, row, col, row, col, { 'inserted ', 'two lines ' })
    api.nvim_buf_set_text(buf, row, col, row + 1, 10, { '' })
    api.nvim_buf_set_lines(buf, row, row, true, { 'a new line' })
    api.nvim_buf_set_lines(buf, row, row + 1, true, {})
  end
  assert(vim.deep_equal(before, api.nvim_buf_get_extmark_by_id(buf, ns, ids[NMARK], {})))
end)

-- A deleted range takes its marks into the undo entry; `:undo` puts them
-- back. Undo is on for this phase only.
phase('undo', function()
  reseed(37)
  vim.bo[buf].undolevels = 1000
  local count = #api.nvim_buf_get_extmarks(buf, ns, 0, -1, {})
  for _ = 1, 800 * SCALE do
    local row = rnd(NLINE - 6) - 1
    api.nvim_buf_set_lines(buf, row, row + 5, true, {})
    vim.cmd('silent undo')
  end
  vim.bo[buf].undolevels = -1
  assert(count == #api.nvim_buf_get_extmarks(buf, ns, 0, -1, {}))
end)

-- Every redraw starts the window somewhere new, so each one walks the tree
-- from a seek rather than from where the last left off.
phase('decor', function()
  reseed(41)
  for _ = 1, 300 * SCALE do
    api.nvim_win_set_cursor(0, { rnd(NLINE - 60), 0 })
    vim.cmd('normal! zt')
    vim.cmd('redraw!')
  end
end)

-- Half the marks, in an order that hops across the tree (7919 is prime to
-- NMARK, so the stride visits each index once).
phase('del', function()
  local gone = 0
  for i = 1, NMARK / 2 do
    local k = i * 7919 % NMARK + 1
    if api.nvim_buf_del_extmark(buf, ns, ids[k]) then
      gone = gone + 1
    end
  end
  assert(gone == NMARK / 2)
end)

phase('clear', function()
  api.nvim_buf_clear_namespace(buf, ns, 0, -1)
  assert(#api.nvim_buf_get_extmarks(buf, ns, 0, -1, { limit = 1 }) == 0)
end)

io.stdout:write(table.concat(out, '\n') .. '\n')
