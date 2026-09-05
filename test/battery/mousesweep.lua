-- mousesweep -- the sixteenth baselined differential.  Driven by
-- mousesweep.sh, which builds the sandbox, pins $HOME/
-- $TMPDIR/$PATH and does the scrubs only the shell can see.  Read that
-- header first.
--
-- The subsystem is mouse.rs: `do_mouse` (695 lines), `jump_to_mouse`
-- (370), `ins_mouse`, `do_mousescroll` / `ins_mousescroll` /
-- `do_mousescroll_horiz`, `mouse_comp_pos`, `vcol2col`,
-- `mouse_find_win_inner` / `_outer` / `mouse_find_grid_win`,
-- `mouse_check_grid`, `call_click_def_func`, `do_popup` and
-- `f_getmousepos`.  Before this oracle mouse input appeared in NO
-- differential at all: one `nvim_input_mouse` in evalsweep, one
-- `getmousepos` in keysweep, and zero in scrsweep and opsweep (B19
-- survey S3, hole 2).
--
-- THE DESIGN, in one paragraph.  A mouse key is dispatched by the main
-- input loop, so nothing here can run in this `-l` process: every
-- section starts its own `--embed` child over `jobstart(rpc = true)`,
-- pushes events with `nvim_input_mouse` / `nvim_input`, and reads the
-- answer back with `nvim_exec_lua`.  The answer is the editor's own
-- state -- mode, cursor, the Visual anchor, `winsaveview()`,
-- `getmousepos()`, the window layout, a register where a selection is
-- yanked -- and `screenstring()` rows only where state cannot answer.
-- `nvim_input` and `nvim_input_mouse` are FAST calls and are dispatched
-- even while the child is inside a modal loop; `nvim_exec_lua` is
-- deferred to the main loop and is NOT.
--
-- Sections:
--   s1  buttons     press/drag/release x 5 buttons x 9 modifier sets
--   s2  multiclick  2/3/4-click, the classes they select, the reset rule
--   s3  mouseopt    every 'mouse' value against every mode
--   s4  model       'mousemodel' extend/popup/popup_setpos + the PopUp menu
--   s5  wheel       wheel and horizontal wheel x 'mousescroll'
--   s6  drag        status line / separator / tabline drag-resize, and
--                   the right drag ONTO one, which is the only reader of
--                   `jump_to_mouse`'s `on_status_line`
--   s7  clickdef    %@Func@ regions reached by a real click
--   s8  columns     fold, sign, number and 'statuscolumn' clicks
--   s9  modes       Insert, Visual, Select, operator-pending, cmdline
--   s10 floats      into / through a float, focusable, zindex, border
--   s11 mousepos    getmousepos() and the cursor over a grid of columns
--   s12 multigrid   ext_multigrid grid arguments, UI attached raw
--   s13 gestures    middle paste, CTRL-click, drag-scroll, focus, moves
--   s91 crashprobe  the inputs that may kill the editor, one child each
--
-- Every section ends with a `## <name> rows=N` line.  A sweep that goes
-- silently empty otherwise looks exactly like a healthy one.
--
-- DETERMINISM.  `'mousetime'` is compared against the WALL CLOCK, so
-- every case pins it -- 0 (no click continues the previous one) or
-- 100000 (every click does).  Cases that must start a fresh click count
-- say so explicitly rather than relying on elapsed time.

local work = assert(os.getenv('MOUSE_WORK'), 'MOUSE_WORK unset')
local runtime = os.getenv('VIMRUNTIME') or ''
local script = debug.getinfo(1, 'S').source:sub(2)

local only = os.getenv('MOUSESWEEP_ONLY')
if only == '' then
  only = nil
end
local trace = os.getenv('MOUSESWEEP_TRACE') == '1'

io.stdout:setvbuf('line')

local rows = 0
local function emit(...)
  rows = rows + 1
  io.write(table.concat({ ... }, ' '), '\n')
end

-- --------------------------------------------------------------- scrub

local function scrub(text)
  text = tostring(text)
  text = text:gsub(vim.pesc(work), '<WORK>')
  text = text:gsub(vim.pesc(script), '<SCRIPT>')
  if runtime ~= '' then
    text = text:gsub(vim.pesc(runtime), '<RT>')
  end
  text = text:gsub('nvim%.%d+%.%d+', 'nvim.<PID>.<SEQ>')
  text = text:gsub('nvim%.[%w_.-]+/[%w]+', 'nvim.<U>/<T>')
  return text
end

local function cap(text, limit)
  limit = limit or 400
  if #text <= limit then
    return text
  end
  return text:sub(1, limit) .. string.format('...<+%d>', #text - limit)
end

--- Escape to one printable line.  `mode(1)` answers a raw CTRL-V for
--- blockwise Visual and the fixture carries multibyte text.
local function esc(bytes)
  return (tostring(bytes):gsub('[%c\128-\255\\]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

local SEEN = {}
local function label_once(label)
  if SEEN[label] then
    emit('!!', 'DUPLICATE', 'LABEL', label)
  end
  SEEN[label] = true
  return label
end

-- -------------------------------------------------------------- struct

local structfd =
  assert(io.open(assert(os.getenv('MOUSE_STRUCT'), 'MOUSE_STRUCT unset'), 'w'))

local function q(str)
  return '"'
    .. (str:gsub('[%c"\\\128-\255]', function(c)
      return string.format('\\x%02x', c:byte())
    end))
    .. '"'
end

local canon
function canon(value)
  local t = type(value)
  if t == 'string' then
    return q(scrub(value))
  elseif t == 'number' then
    if value == math.floor(value) and math.abs(value) < 2 ^ 53 then
      return string.format('%d', value)
    end
    return string.format('%.17g', value)
  elseif t ~= 'table' then
    return q(tostring(value))
  end
  local n = 0
  for _ in pairs(value) do
    n = n + 1
  end
  if n == #value then
    local parts = {}
    for i, item in ipairs(value) do
      parts[i] = canon(item)
    end
    return '[' .. table.concat(parts, ',') .. ']'
  end
  local keys = {}
  for k in pairs(value) do
    keys[#keys + 1] = tostring(k)
  end
  table.sort(keys)
  local parts = {}
  for _, k in ipairs(keys) do
    parts[#parts + 1] = q(k) .. ':' .. canon(value[k] == nil and value[tonumber(k)] or value[k])
  end
  return '{' .. table.concat(parts, ',') .. '}'
end

local function struct(label, value)
  structfd:write(label, '\t', canon(value), '\n')
end

--- Normalise an error to its message.  A pcall against an API function
--- prefixes the Lua source position, which is a line number in THIS
--- file and would re-baseline the whole artifact on any edit above.
local function errtext(res)
  local s = tostring(res)
  s = s:gsub('\nstack traceback:.*$', '')
  s = s:gsub('^[^\n]-mousesweep%.lua:%d+: ', '')
  s = s:gsub('^%[string "[^"]*"%]:%d+: ', '')
  s = s:gsub('\r?\n', ' | ')
  return cap(scrub(s))
end

local function ins(value)
  return (vim.inspect(value, { newline = ' ', indent = '' }))
end

--- A dying child's stderr, reduced to the part that is a FACT about the
--- editor rather than about this machine.  A non-unwinding panic prints
--- a full backtrace whether or not `RUST_BACKTRACE` is set, and that is
--- fifty lines of addresses, a thread id, a rustc hash and the absolute
--- source paths of the toolchain -- none of it reproducible.  The
--- source LINE is dropped as well: it is real information, but it moves
--- whenever anything above it in the file moves, and a differential
--- that re-baselines on an unrelated edit is worse than useless.
local function said(lines)
  local out = {}
  for _, line in ipairs(lines or {}) do
    if line:match('^stack backtrace:') or line:match('^%s*%d+:%s+0x') then
      break
    end
    if not line:match('^note: run with') and not line:match('^%s*$') then
      line = line:gsub('%((%d+)%)', '(<TID>)')
      line = line:gsub('0x%x+', '<ADDR>')
      line = line:gsub('(%.rs):%d+:%d+', '%1:<L>')
      line = line:gsub('/rustc/%w+/', '/rustc/<HASH>/')
      out[#out + 1] = scrub(line)
    end
  end
  return out
end

--- Serialise a small option table back into Lua source, so the parent
--- can hand `RESET`/`ANS` their arguments without a second roundtrip.
local function tolua(value)
  local t = type(value)
  if t == 'nil' then
    return 'nil'
  elseif t == 'boolean' or t == 'number' then
    return tostring(value)
  elseif t == 'string' then
    return string.format('%q', value)
  end
  local parts = {}
  if #value > 0 then
    for _, v in ipairs(value) do
      parts[#parts + 1] = tolua(v)
    end
  else
    local keys = {}
    for k in pairs(value) do
      keys[#keys + 1] = k
    end
    table.sort(keys)
    for _, k in ipairs(keys) do
      parts[#parts + 1] = string.format('[%q]=%s', k, tolua(value[k]))
    end
  end
  return '{' .. table.concat(parts, ',') .. '}'
end

-- ------------------------------------------------------------ sections

local secrows = 0
local function section(name, fn)
  if only and not name:match(only) then
    return
  end
  if trace then
    io.stderr:write('== ' .. name .. '\n')
  end
  secrows = rows
  emit('##', name)
  local ok, err = pcall(fn)
  if not ok then
    emit('##', name, 'RAISED', esc(scrub(errtext(err))))
  end
  emit('##', name, string.format('rows=%d', rows - secrows - 1))
end

-- ================================================================ child
-- Every section drives one `--embed` child.  The child is where the
-- main input loop lives, and the main input loop is the only thing that
-- dispatches a mouse key.
-- ====================================================================

--- The fixture.  Line 3 is leading-tab, line 4 two-byte characters,
--- line 5 double-width, line 8 long enough for `'nowrap'` scrolling.
local FIXTURE = {
  'alpha beta gamma delta',
  'one,two;three.four+five',
  '\ttab\tsep\tline',
  '\u{03b1}\u{03b1}\u{03b1} \u{03b2}\u{03b2}\u{03b2} \u{03b3}\u{03b3}\u{03b3}',
  '\u{65e5}\u{672c}\u{8a9e} \u{30c6}\u{30ad}\u{30b9}\u{30c8} \u{5e45}',
  'short',
  '',
  'The quick brown fox jumps over the lazy dog and then keeps running well past the eightieth column of the screen.',
  'x',
}

--- Installed once per child.  `RESET` is the world between cases and
--- `ANS` is the one answer shape; keeping both in the child means one
--- RPC roundtrip per case instead of a dozen.
local PRELUDE = [[
_G.MSFIX = ]] .. tolua(FIXTURE) .. [[

_G.MSCLICKS = {}
function _G.RESET(o)
  o = o or {}
  pcall(vim.cmd, 'silent! tabonly!')
  for _, w in ipairs(vim.api.nvim_list_wins()) do
    if vim.api.nvim_win_get_config(w).relative ~= '' then
      pcall(vim.api.nvim_win_close, w, true)
    end
  end
  pcall(vim.cmd, 'silent! only!')
  pcall(vim.cmd, 'silent! stopinsert')
  if vim.fn.mode() ~= 'n' then pcall(vim.cmd, 'silent! normal! \27') end
  vim.o.lines = 24
  vim.o.columns = 80
  vim.o.laststatus = o.laststatus or 2
  vim.o.showtabline = o.showtabline or 1
  vim.o.ruler = false
  vim.o.showcmd = false
  vim.o.report = 9999
  vim.o.more = false
  vim.o.shortmess = 'aoOtTIcCF'
  vim.o.statusline = ''
  vim.o.tabline = ''
  vim.o.winbar = ''
  vim.o.statuscolumn = ''
  vim.o.mouse = o.mouse or 'a'
  vim.o.mousemodel = o.mousemodel or 'extend'
  vim.o.mousescroll = o.mousescroll or 'ver:3,hor:6'
  vim.o.mousetime = o.mousetime or 0
  vim.o.mousemoveevent = false
  vim.o.scrolloff = o.scrolloff or 0
  vim.o.sidescrolloff = o.sidescrolloff or 0
  vim.o.sidescroll = o.sidescroll or 0
  vim.o.virtualedit = o.virtualedit or ''
  vim.o.selection = o.selection or 'inclusive'
  vim.o.selectmode = o.selectmode or ''
  vim.o.startofline = true
  vim.o.equalalways = true
  vim.o.splitbelow = false
  vim.o.splitright = false
  vim.o.winminheight = 1
  vim.o.winminwidth = 1
  vim.o.tabstop = 8
  vim.o.list = false
  vim.o.hidden = true
  vim.o.swapfile = false
  vim.o.foldenable = true
  vim.wo.wrap = (o.wrap ~= false)
  vim.wo.number = o.number or false
  vim.wo.relativenumber = o.relativenumber or false
  vim.wo.numberwidth = 4
  vim.wo.signcolumn = o.signcolumn or 'auto'
  vim.wo.foldcolumn = o.foldcolumn or '0'
  vim.wo.foldmethod = 'manual'
  vim.wo.conceallevel = 0
  vim.wo.cursorline = false
  vim.fn.sign_unplace('')
  pcall(vim.cmd, 'silent! normal! zE')
  local buf = vim.api.nvim_create_buf(true, true)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, o.lines and o.lines or _G.MSFIX)
  vim.api.nvim_win_set_buf(0, buf)
  vim.api.nvim_win_set_cursor(0, {1, 0})
  _G.MSCLICKS = {}
  vim.g.picked = 'none'
  vim.fn.setreg('"', '')
  vim.cmd('redraw')
end

function _G.ANS(o)
  o = o or {}
  local r = {}
  local ok, m = pcall(vim.fn.mode, 1)
  r.md = ok and m or '?'
  local w = vim.api.nvim_get_current_win()
  r.win = w
  local okc, c = pcall(vim.api.nvim_win_get_cursor, w)
  r.cur = okc and c or {-1, -1}
  local v = vim.fn.getpos('v')
  r.vs = {v[2], v[3], v[4]}
  local vw = vim.fn.winsaveview()
  r.vw = {vw.topline, vw.leftcol, vw.curswant, vw.skipcol, vw.topfill, vw.coladd}
  local mp = vim.fn.getmousepos()
  r.mp = {mp.screenrow, mp.screencol, mp.winid, mp.winrow, mp.wincol, mp.line, mp.column, mp.coladd}
  if o.wins then
    r.wl = {}
    for _, id in ipairs(vim.api.nvim_list_wins()) do
      local p = vim.api.nvim_win_get_position(id)
      r.wl[#r.wl + 1] = {id, vim.api.nvim_win_get_width(id), vim.api.nvim_win_get_height(id), p[1], p[2]}
    end
    r.tab = {vim.fn.tabpagenr(), vim.fn.tabpagenr('$')}
  end
  if o.clicks then r.ck = _G.MSCLICKS end
  if o.rows then
    r.sc = {}
    for _, row in ipairs(o.rows) do
      local s = {}
      for col = 1, vim.o.columns do s[#s + 1] = vim.fn.screenstring(row, col) end
      r.sc[#r.sc + 1] = row .. ':' .. (table.concat(s):gsub('%s+$', ''))
    end
  end
  if o.q then
    local f, e = load(o.q)
    if not f then r.q = 'LOADERR ' .. tostring(e) else
      -- NOT `okq and res or ...`: a query that legitimately answers
      -- `false` (`'mousemoveevent'` off) would be reported as an error.
      local okq, res = pcall(f)
      if okq then r.q = res else r.q = 'ERR ' .. tostring(res) end
    end
  end
  return r
end
]]

local Child = {}
Child.__index = Child

local function child_start(args)
  local argv = {
    work .. '/bin/nvim',
    '--headless',
    '--embed',
    '-u',
    'NONE',
    '-i',
    'NONE',
    '--cmd',
    'set noswapfile',
  }
  for _, a in ipairs(args or {}) do
    argv[#argv + 1] = a
  end
  local errlines = {}
  local chan = vim.fn.jobstart(argv, {
    rpc = true,
    cwd = work,
    clear_env = true,
    -- Without this the child's dying words are dropped on the floor:
    -- `jobstart` discards stderr it is not asked for, and s91's whole
    -- verdict is "did it die and what did it say".
    on_stderr = function(_, data)
      for _, line in ipairs(data or {}) do
        if line ~= '' then
          errlines[#errlines + 1] = line
        end
      end
    end,
    env = {
      HOME = work .. '/home',
      PATH = work .. '/bin',
      TMPDIR = work .. '/tmp',
      TERM = 'dumb',
      SHELL = '/bin/sh',
      LANG = 'C.UTF-8',
      VIMRUNTIME = runtime,
      NVIM_TEST = '1',
    },
  })
  assert(chan > 0, 'embed child failed to start')
  return setmetatable({ chan = chan, dead = false, err = errlines }, Child)
end

--- Run a chunk in the child.
---
--- A failed `rpcrequest` is TWO different events and conflating them
--- poisons a whole section: a Lua error inside the chunk comes back as
--- an error RESPONSE (the child is fine), while a dead child comes back
--- as a closed channel.  Only the second means dead -- the first draft
--- marked the child dead on any error and one `E490: No fold found` in
--- a setup took sixty-one later cases down with it.
function Child:lua(code)
  if self.dead then
    return 'DEAD'
  end
  local ok, res = pcall(vim.rpcrequest, self.chan, 'nvim_exec_lua', code, {})
  if ok then
    return res
  end
  local text = errtext(res)
  if text:match('closed by the peer') or text:match('[Ii]nvalid channel') or text:match('channel closed') then
    self.dead = true
  end
  return 'RPCERR ' .. text
end

function Child:mouse(button, action, mods, row, col, grid)
  if self.dead then
    return 'DEAD'
  end
  local ok, err =
    pcall(vim.rpcrequest, self.chan, 'nvim_input_mouse', button, action, mods or '', grid or 0, row, col)
  if ok then
    return 'ok'
  end
  return 'ERR ' .. errtext(err)
end

function Child:key(keys)
  if self.dead then
    return 'DEAD'
  end
  local ok, err = pcall(vim.rpcrequest, self.chan, 'nvim_input', keys)
  return ok and 'ok' or ('ERR ' .. errtext(err))
end

--- Bring the child back to a KNOWN state and prove that every key sent
--- so far has been consumed.
---
--- This is the whole determinism story and it cost three
--- nondeterministic rows to find.  `nvim_input` / `nvim_input_mouse`
--- are FAST calls that drop bytes straight into the input buffer, while
--- `nvim_exec_lua` is DEFERRED to the main loop -- so a case's `RESET`
--- can run while the previous case's `<Esc>` is still in the typeahead,
--- and that `<Esc>` then lands in the middle of the new case's click
--- chain and resets its click count.  Polling `mode()` is not enough
--- either, because a case that ended in Normal mode leaves nothing for
--- `mode()` to show.  So the flush sends a NUMBERED marker through the
--- same FIFO the keys went through: once `g:mssync` reads back this
--- case's number, every earlier key has been executed, by construction.
function Child:flush()
  self.seq = (self.seq or 0) + 1
  -- Esc leaves Visual, Insert and the command line, and it is also what
  -- breaks `pum_show_popupmenu`'s key loop -- which answers no
  -- `nvim_exec_lua` at all while it is up.  CTRL-C would do the same
  -- and must NOT be used: it sets `got_int`, which then interrupts the
  -- `<Cmd>` below and the marker is never set.
  for _ = 1, 200 do
    if self.dead then
      return false
    end
    -- One Esc is not always enough: a Visual selection STARTED FROM
    -- INSERT MODE (which is exactly what a mouse drag in Insert mode
    -- makes) answers Esc by going back to Insert, so the loop keeps
    -- sending until `mode()` really is `n`.
    self:key('\27')
    self:key('<Cmd>let g:mssync=' .. self.seq .. '<CR>')
    local v = self:lua('return {vim.g.mssync, vim.fn.mode()}')
    if type(v) == 'table' and v[1] == self.seq and v[2] == 'n' then
      return true
    end
    vim.wait(1)
  end
  return false
end

--- A KEYLESS barrier: one deferred request, answered.
---
--- `nvim_exec_lua` is dispatched by the same loop that dispatches keys
--- and the loop takes the typeahead first, so an answer to this proves
--- every key sent before it has been executed.  It must be KEYLESS,
--- and that is not a stylistic preference: `do_mouse` coalesces drags
--- by calling `vpeekc()` and, if anything at all is queued,
--- `safe_vgetc()` -- which decodes the next key and RESETS `mod_mask`
--- for the drag it is in the middle of handling.  A `<Cmd>` marker
--- injected after a drag therefore strips the drag's own modifier,
--- turning an Alt-drag's blockwise selection into a charwise one, and
--- whether it does so depends on whether the marker had arrived yet.
--- That was the last nondeterministic row in this sweep.
function Child:barrier()
  return self:lua('return 1') == 1
end

--- One full turn of the child's state machine, with no state change.
---
--- `RESET`/`setup` run as a DEFERRED event, so their effects that are
--- only applied when control returns to the main loop -- `startinsert`
--- setting `restart_edit`, a pending redraw, a window layout that has
--- not been drawn yet -- are not in force when the case's first mouse
--- event arrives.  Sending one `<Cmd>` marker and waiting for it proves
--- the loop has turned.
function Child:settle()
  self.tick = (self.tick or 0) + 1
  self:key('<Cmd>let g:mstick=' .. self.tick .. '<CR>')
  for _ = 1, 200 do
    if self.dead then
      return false
    end
    if self:lua('return vim.g.mstick') == self.tick then
      return true
    end
    vim.wait(1)
  end
  return false
end

--- Wait for a deferred mode change.  `vim.cmd('startinsert')` from a
--- deferred event only sets `restart_edit`; Insert mode begins when the
--- state machine next turns, which can be after the case's first mouse
--- event has already been queued.
function Child:want(mode)
  for _ = 1, 500 do
    if self.dead then
      return false
    end
    if self:lua('return vim.fn.mode()') == mode then
      return true
    end
    vim.wait(1)
  end
  return false
end

--- `jobstop` alone is not enough: the child has to be reaped, or it
--- outlives this process holding $WORK as its cwd while the next run's
--- `rm -rf` races it (the cmdsweep s18 lesson).
function Child:stop()
  pcall(vim.fn.jobstop, self.chan)
  pcall(vim.fn.jobwait, { self.chan }, 10000)
end

--- Boot a child and install the prelude.
local function child_new(args)
  local c = child_start(args)
  c:lua(PRELUDE)
  return c
end

-- ============================================================== the case
-- ONE case: reset the world, run a setup chunk, push a list of events,
-- read the answer back, then leave the child in a known state.
--
-- An event is one of
--    {'m', button, action, mods, row, col[, grid]}
--    {'k', '<Esc>'}          -- a FAST call; works inside a modal loop
--    {'l', 'lua chunk'}      -- deferred, like every other query
-- ====================================================================

local CUR

--- Whether the child is made to finish each event before the next is
--- sent.  ON everywhere except s4, where a case parks the child inside
--- `pum_show_popupmenu` and `settle` would wait for an answer that
--- cannot come until the menu is down.
local SYNC = true

--- `sync` makes the child finish each event before the next is sent.
--- A press and the drag that follows it are two separate keys, and
--- pushing both into the input buffer before the editor has looked at
--- either is not what a user does -- it is also the last piece of
--- nondeterminism in this sweep: whether the status line was grabbed
--- depended on whether the press had been dispatched when the drag
--- arrived, and s6's answers flipped between runs because of it.  It is
--- OFF where a case opens a modal menu, because `settle` needs
--- `nvim_exec_lua` and `pum_show_popupmenu` does not serve it.
local function run_events(c, events, sync)
  local notes = nil
  for i, e in ipairs(events or {}) do
    local res
    if e[1] == 'm' then
      res = c:mouse(e[2], e[3], e[4], e[5], e[6], e[7])
    elseif e[1] == 'k' then
      res = c:key(e[2])
    elseif e[1] == 'w' then
      vim.wait(e[2])
      res = 'ok'
    elseif e[1] == 'l' then
      res = c:lua(e[2])
      res = (res == vim.NIL or res == nil) and 'ok' or tostring(res)
    else
      res = 'BADEVENT'
    end
    if res ~= 'ok' then
      notes = notes or {}
      notes[#notes + 1] = i .. ':' .. tostring(res)
    end
    if sync == 'wait' then
      -- The weaker spelling, for the sections that cannot ask the child
      -- anything: while a `'mousemodel'=popup` menu is up there is no
      -- answer to a barrier, and a crashprobe child may be dead.  A
      -- real pause is all that is left, and it is enough because the
      -- child is otherwise idle.
      vim.wait(10)
    elseif sync then
      -- A drag is the one event that peeks at what comes next, so give
      -- the child a moment to dispatch it with an EMPTY queue before
      -- the barrier's own event is added to that queue.
      if e[1] == 'm' and e[3] == 'drag' then
        vim.wait(5)
      end
      if not c:barrier() then
        notes = notes or {}
        notes[#notes + 1] = i .. ':BARRIER-FAILED'
      end
    end
  end
  return notes
end

local function case(label, opts)
  opts = opts or {}
  local c = opts.child or CUR
  label_once(label)
  local notes = nil
  if not c:flush() then
    notes = { 'FLUSH-FAILED' }
  end
  -- The setup is wrapped so that a case whose fixture is wrong reports
  -- one bad row instead of an unexplained answer: `c:lua` would return
  -- the error, but the RESET before it has already run and the case
  -- would otherwise look merely surprising.
  local body = (opts.noreset and '' or ('RESET(' .. tolua(opts.reset or {}) .. ')\n'))
    .. (opts.setup or '')
  local status = c:lua('local ok, err = pcall(function()\n' .. body .. '\nend)\nreturn ok and "ok" or ("SETUP: " .. tostring(err))')
  if status ~= 'ok' then
    notes = notes or {}
    notes[#notes + 1] = tostring(status)
  end
  if opts.want and not c:want(opts.want) then
    notes = notes or {}
    notes[#notes + 1] = 'WANT-' .. opts.want .. '-FAILED'
  end
  if not c:settle() then
    notes = notes or {}
    notes[#notes + 1] = 'SETTLE-FAILED'
  end
  local more = run_events(c, opts.events, opts.sync == nil and SYNC or opts.sync)
  if more then
    notes = notes or {}
    for _, n in ipairs(more) do
      notes[#notes + 1] = n
    end
  end
  local ans = c:lua('return ANS(' .. tolua(opts.ans or {}) .. ')')
  if notes then
    if type(ans) == 'table' then
      ans.ev = notes
    else
      ans = { bad = tostring(ans), ev = notes }
    end
  end
  emit(label, '=', esc(scrub(ins(ans))))
  struct(label, ans)
  return ans
end

local function ask(label, value)
  label_once(label)
  emit(label, 'A', esc(scrub(ins(value))))
  struct(label, value)
end

-- ==================================================================== s1
-- Buttons, actions and modifiers.  Every spelling `do_mouse`'s
-- which_button / is_click / is_drag switch can reach, against one fixed
-- buffer, with the click count pinned OFF (`mousetime = 0`) so that no
-- case inherits the previous one's counter.
-- ====================================================================

local BUTTONS = { 'left', 'right', 'middle', 'x1', 'x2' }
local MODS = { '', 'S', 'C', 'A', 'M', 'D', 'SC', 'SA', 'SCA' }

section('s1-buttons', function()
  CUR = child_new()

  -- Each button through each sequence shape.  A drag with no press and
  -- a release with no press are both legal inputs and both reach
  -- `do_mouse` with `is_click` false.
  local SEQS = {
    { 'press', { { 'press' } } },
    { 'press-release', { { 'press' }, { 'release' } } },
    { 'press-drag-release', { { 'press' }, { 'drag' }, { 'release' } } },
    { 'drag-only', { { 'drag' } } },
    { 'release-only', { { 'release' } } },
    { 'press-drag-drag', { { 'press' }, { 'drag' }, { 'drag' } } },
  }
  for _, b in ipairs(BUTTONS) do
    for _, s in ipairs(SEQS) do
      local events = {}
      for i, step in ipairs(s[2]) do
        -- The drags walk down-right so a Visual selection has a shape.
        events[#events + 1] = { 'm', b, step[1], '', 1 + (i - 1), 4 + (i - 1) * 3 }
      end
      case('s1/seq/' .. b .. '/' .. s[1], { events = events })
    end
  end

  -- Modifiers on a click and on a drag.  Shift-left is "extend the
  -- selection", CTRL-left is a tag jump, and both are decided before
  -- `jump_to_mouse` ever runs.
  for _, b in ipairs(BUTTONS) do
    for _, m in ipairs(MODS) do
      local name = m == '' and 'none' or m
      case('s1/mods/' .. b .. '/' .. name, {
        events = {
          { 'm', b, 'press', m, 2, 6 },
          { 'm', b, 'release', m, 2, 6 },
        },
      })
    end
  end
  for _, m in ipairs(MODS) do
    local name = m == '' and 'none' or m
    case('s1/dragmods/' .. name, {
      events = {
        { 'm', 'left', 'press', m, 1, 2 },
        { 'm', 'left', 'drag', m, 3, 9 },
        { 'm', 'left', 'release', m, 3, 9 },
      },
    })
  end

  -- Where the click lands: every interesting column of the fixture,
  -- including past the end of a line, on a tab, inside a multibyte and
  -- a double-width character, and on the empty line.
  for row = 0, 8 do
    for _, col in ipairs({ 0, 1, 3, 5, 8, 12, 21, 40, 79 }) do
      case(string.format('s1/pos/r%d/c%02d', row, col), {
        events = {
          { 'm', 'left', 'press', '', row, col },
          { 'm', 'left', 'release', '', row, col },
        },
      })
    end
  end

  -- Below the last line, on the status line, and on the command line.
  for _, row in ipairs({ 9, 15, 21, 22, 23 }) do
    case('s1/below/r' .. row, {
      events = {
        { 'm', 'left', 'press', '', row, 5 },
        { 'm', 'left', 'release', '', row, 5 },
      },
      ans = { wins = true },
    })
  end

  CUR:stop()
end)

-- ==================================================================== s2
-- Multi-click.  `'mousetime'` decides whether a click continues the
-- previous one, and the count decides word / line / block selection.
-- Both spellings are pinned: 100000 makes every click a continuation,
-- 0 makes none of them one, and the DIFFERENCE is the counter.
-- ====================================================================

section('s2-multiclick', function()
  CUR = child_new()

  local function clicks(n, row, col, button)
    local ev = {}
    for _ = 1, n do
      ev[#ev + 1] = { 'm', button or 'left', 'press', '', row, col }
      ev[#ev + 1] = { 'm', button or 'left', 'release', '', row, col }
    end
    return ev
  end

  for _, mt in ipairs({ 0, 100000 }) do
    for n = 1, 5 do
      for _, b in ipairs({ 'left', 'right', 'middle' }) do
        case(string.format('s2/count/mt%d/%s/%d', mt, b, n), {
          reset = { mousetime = mt },
          events = clicks(n, 1, 6, b),
        })
      end
    end
  end

  -- What a double click selects, by character class, and what a triple
  -- and a quadruple click select over the same column.
  for n = 2, 4 do
    for _, spot in ipairs({
      { 'word', 0, 2 },
      { 'wordend', 0, 4 },
      { 'space', 0, 5 },
      { 'punct', 1, 3 },
      { 'punctrun', 1, 11 },
      { 'tab', 2, 0 },
      { 'aftertab', 2, 8 },
      { 'multibyte', 3, 2 },
      { 'doublewide', 4, 2 },
      { 'past-eol', 5, 20 },
      { 'empty', 6, 0 },
      { 'single', 8, 0 },
    }) do
      case(string.format('s2/class/%d/%s', n, spot[1]), {
        reset = { mousetime = 100000 },
        events = clicks(n, spot[2], spot[3]),
        ans = { q = 'vim.cmd("silent! normal! y") return {vim.fn.getreg(\'"\'), vim.fn.getregtype(\'"\')}' },
      })
    end
  end

  -- The reset rules.  A click at a different position, a different
  -- button in between and an intervening key all decide whether the
  -- counter continues -- and none of them is the clock.
  case('s2/reset/samepos', {
    reset = { mousetime = 100000 },
    events = clicks(2, 1, 6),
  })
  case('s2/reset/movedcol', {
    reset = { mousetime = 100000 },
    events = {
      { 'm', 'left', 'press', '', 1, 6 },
      { 'm', 'left', 'release', '', 1, 6 },
      { 'm', 'left', 'press', '', 1, 12 },
      { 'm', 'left', 'release', '', 1, 12 },
    },
  })
  case('s2/reset/movedrow', {
    reset = { mousetime = 100000 },
    events = {
      { 'm', 'left', 'press', '', 1, 6 },
      { 'm', 'left', 'release', '', 1, 6 },
      { 'm', 'left', 'press', '', 2, 6 },
      { 'm', 'left', 'release', '', 2, 6 },
    },
  })
  case('s2/reset/otherbutton', {
    reset = { mousetime = 100000 },
    events = {
      { 'm', 'left', 'press', '', 1, 6 },
      { 'm', 'left', 'release', '', 1, 6 },
      { 'm', 'right', 'press', '', 1, 6 },
      { 'm', 'right', 'release', '', 1, 6 },
      { 'm', 'left', 'press', '', 1, 6 },
      { 'm', 'left', 'release', '', 1, 6 },
    },
  })
  case('s2/reset/interveningkey', {
    reset = { mousetime = 100000 },
    events = {
      { 'm', 'left', 'press', '', 1, 6 },
      { 'm', 'left', 'release', '', 1, 6 },
      { 'k', 'j' },
      { 'm', 'left', 'press', '', 1, 6 },
      { 'm', 'left', 'release', '', 1, 6 },
    },
  })
  case('s2/reset/eightclicks', {
    reset = { mousetime = 100000 },
    events = clicks(8, 1, 6),
  })

  -- A double click that then drags extends by whole words; a triple
  -- click that drags extends by whole lines.
  for n = 2, 3 do
    local ev = clicks(n - 1, 1, 6)
    ev[#ev + 1] = { 'm', 'left', 'press', '', 1, 6 }
    ev[#ev + 1] = { 'm', 'left', 'drag', '', 1, 18 }
    ev[#ev + 1] = { 'm', 'left', 'release', '', 1, 18 }
    case('s2/dragextend/' .. n, { reset = { mousetime = 100000 }, events = ev })
  end

  -- `'selection'` changes where a Visual selection ends.
  for _, sel in ipairs({ 'inclusive', 'exclusive', 'old' }) do
    case('s2/selection/' .. sel, {
      reset = { mousetime = 100000, selection = sel },
      events = clicks(2, 0, 2),
      ans = { q = 'vim.cmd("silent! normal! y") return vim.fn.getreg(\'"\')' },
    })
  end

  CUR:stop()
end)

-- ==================================================================== s3
-- The `'mouse'` option.  NOTE what it does and does not gate: a real
-- terminal only sends mouse codes when `setmouse()` turned them on, and
-- `nvim_input_mouse` bypasses that entirely, so a click still moves the
-- cursor with `'mouse'` empty.  What `'mouse'` gates INSIDE the editor
-- is `ui_mouse_has(MOUSE_VISUAL)` in `do_mouse` -- whether a drag may
-- start Visual mode -- plus the `h` flag against a help buffer.
-- ====================================================================

local HELPBUF = [[
local hb = vim.api.nvim_create_buf(false, true)
vim.api.nvim_buf_set_lines(hb, 0, -1, false, {
  '*help.txt*  a synthetic help buffer', 'first line', 'second line',
  'third line', 'fourth line', 'fifth line', 'sixth line', 'seventh line',
})
vim.api.nvim_win_set_buf(0, hb)
vim.bo[hb].buftype = 'help'
vim.cmd('redraw')
]]

section('s3-mouseopt', function()
  CUR = child_new()

  local VALUES = { '', 'n', 'v', 'i', 'c', 'h', 'r', 'a', 'nv', 'nvi', 'ni', 'ah' }
  for _, mv in ipairs(VALUES) do
    local name = mv == '' and 'empty' or mv
    -- A plain click.
    case('s3/' .. name .. '/click', {
      reset = { mouse = mv },
      events = {
        { 'm', 'left', 'press', '', 1, 6 },
        { 'm', 'left', 'release', '', 1, 6 },
      },
    })
    -- A drag, which is the arm `'mouse'` really decides.
    case('s3/' .. name .. '/drag', {
      reset = { mouse = mv },
      events = {
        { 'm', 'left', 'press', '', 1, 4 },
        { 'm', 'left', 'drag', '', 2, 10 },
        { 'm', 'left', 'release', '', 2, 10 },
      },
    })
    -- A right drag, the other `mouse_can_visual` arm.
    case('s3/' .. name .. '/rightdrag', {
      reset = { mouse = mv },
      events = {
        { 'm', 'right', 'press', '', 1, 4 },
        { 'm', 'right', 'drag', '', 2, 10 },
        { 'm', 'right', 'release', '', 2, 10 },
      },
    })
    -- In Insert mode.
    case('s3/' .. name .. '/insert', {
      reset = { mouse = mv },
      setup = 'vim.cmd("startinsert") vim.cmd("redraw")',
      want = 'i',
      events = {
        { 'm', 'left', 'press', '', 1, 4 },
        { 'm', 'left', 'drag', '', 2, 10 },
        { 'm', 'left', 'release', '', 2, 10 },
      },
    })
    -- In a help buffer, which is the only thing the `h` flag reads.
    case('s3/' .. name .. '/help', {
      reset = { mouse = mv },
      -- A SYNTHETIC help buffer, not `:help`.  `'buftype'` = help is
      -- what sets `b_help`, which is all the `h` flag reads, and the
      -- real thing pulls in $VIMRUNTIME, a ftplugin and a syntax file
      -- -- three things a differential should not depend on.
      setup = HELPBUF,
      events = {
        { 'm', 'left', 'press', '', 3, 4 },
        { 'm', 'left', 'drag', '', 5, 10 },
        { 'm', 'left', 'release', '', 5, 10 },
      },
      ans = { q = 'return {vim.bo.buftype, vim.bo.filetype, vim.bo.buflisted}' },
    })
    -- On the command line.
    case('s3/' .. name .. '/cmdline', {
      reset = { mouse = mv },
      events = {
        { 'k', ':abcdefghij' },
        { 'm', 'left', 'press', '', 23, 4 },
        { 'm', 'left', 'release', '', 23, 4 },
      },
      ans = { q = 'return {vim.fn.mode(), vim.fn.getcmdline(), vim.fn.getcmdpos()}' },
    })
  end

  ask('s3/optvalues', CUR:lua([[
    local out = {}
    for _, v in ipairs({'', 'a', 'nvi', 'h', 'r', 'nvichr'}) do
      local ok = pcall(function() vim.o.mouse = v end)
      out[#out+1] = v .. '=' .. tostring(ok) .. '/' .. vim.o.mouse
    end
    local bad = {}
    for _, v in ipairs({'x', 'nn', 'A', 'n,v'}) do
      local ok, e = pcall(function() vim.o.mouse = v end)
      bad[#bad+1] = v .. '=' .. tostring(ok) .. '/' .. (ok and '' or tostring(e):gsub('.*: ', ''))
    end
    vim.o.mouse = 'a'
    return {out, bad}
  ]]))

  CUR:stop()
end)

-- ==================================================================== s4
-- `'mousemodel'`.  `popup` and `popup_setpos` reach `do_popup` ->
-- `show_popupmenu` -> `pum_show_popupmenu`, WHICH RUNS ITS OWN
-- `vgetc()` LOOP WITH NO `K_EVENT` ARM: while the menu is up the child
-- answers no `nvim_exec_lua` at all, so the menu is driven with
-- `nvim_input` (a fast call) and only queried once it is down.
--
-- The runtime already defines a fifteen-entry `PopUp` menu (Open in web
-- browser, Inspect, Go to definition, ...).  It must be removed first,
-- or `j<CR>` picks `gx` and shells out.
-- ====================================================================

local POPUP_SETUP = [[
pcall(vim.cmd, 'silent! aunmenu PopUp')
vim.cmd('anoremenu PopUp.Alpha :let g:picked="alpha"<CR>')
vim.cmd('anoremenu PopUp.Beta :let g:picked="beta"<CR>')
vim.cmd('anoremenu PopUp.-sep- :<CR>')
vim.cmd('anoremenu PopUp.Gamma :let g:picked="gamma"<CR>')
vim.cmd('vnoremenu PopUp.OnlyVisual :<C-u>let g:picked="visualonly"<CR>')
vim.g.picked = 'none'
vim.cmd('redraw')
]]

section('s4-model', function()
  CUR = child_new()
  SYNC = 'wait'

  for _, mm in ipairs({ 'extend', 'popup', 'popup_setpos' }) do
    -- Left, middle and right, click and drag.  The right button is the
    -- one the model switches; the others must be unaffected.
    for _, b in ipairs({ 'left', 'right', 'middle' }) do
      case(string.format('s4/%s/%s/click', mm, b), {
        reset = { mousemodel = mm },
        setup = POPUP_SETUP,
        events = {
          { 'm', b, 'press', '', 2, 6 },
          { 'm', b, 'release', '', 2, 6 },
          { 'k', '\27' },
        },
        ans = { q = 'return vim.g.picked' },
      })
      case(string.format('s4/%s/%s/drag', mm, b), {
        reset = { mousemodel = mm },
        setup = POPUP_SETUP,
        events = {
          { 'm', b, 'press', '', 1, 3 },
          { 'm', b, 'drag', '', 2, 9 },
          { 'm', b, 'release', '', 2, 9 },
          { 'k', '\27' },
        },
        ans = { q = 'return vim.g.picked' },
      })
    end

    -- Shift-left is the extend gesture in the `extend` model and a
    -- plain click in the popup models.
    case(string.format('s4/%s/shiftleft', mm), {
      reset = { mousemodel = mm },
      setup = POPUP_SETUP,
      events = {
        { 'm', 'left', 'press', '', 1, 2 },
        { 'm', 'left', 'release', '', 1, 2 },
        { 'm', 'left', 'press', 'S', 3, 9 },
        { 'm', 'left', 'release', 'S', 3, 9 },
        { 'k', '\27' },
      },
      ans = { q = 'return vim.g.picked' },
    })
  end

  -- Navigating the menu.  Every key `pum_menu_key` understands.
  for _, seq in ipairs({
    { 'esc', { '\27' } },
    { 'ctrlc', { '\3' } },
    { 'cr-noselect', { '\r' } },
    { 'j-cr', { 'j', '\r' } },
    { 'jj-cr', { 'j', 'j', '\r' } },
    { 'jjj-cr', { 'j', 'j', 'j', '\r' } },
    { 'jjjj-cr', { 'j', 'j', 'j', 'j', '\r' } },
    { 'jjjjj-cr', { 'j', 'j', 'j', 'j', 'j', '\r' } },
    { 'k-cr', { 'k', '\r' } },
    { 'jjk-cr', { 'j', 'j', 'k', '\r' } },
    { 'down-cr', { '<Down>', '<CR>' } },
    { 'up-cr', { '<Up>', '<CR>' } },
    { 'other-cr', { 'z', 'j', '\r' } },
  }) do
    local ev = {
      { 'm', 'right', 'press', '', 2, 6 },
      { 'm', 'right', 'release', '', 2, 6 },
      { 'w', 60 },
    }
    for _, k in ipairs(seq[2]) do
      ev[#ev + 1] = { 'k', k }
    end
    ev[#ev + 1] = { 'k', '\27' }
    case('s4/menukey/' .. seq[1], {
      reset = { mousemodel = 'popup' },
      setup = POPUP_SETUP,
      events = ev,
      ans = { q = 'return vim.g.picked' },
    })
  end

  -- Selecting with the mouse inside the menu, and the wheel keys the
  -- loop maps onto j/k.
  for _, spot in ipairs({ { 'row3', 3, 8 }, { 'row4', 4, 8 }, { 'row5', 5, 8 }, { 'row9', 9, 8 }, { 'offmenu', 3, 60 } }) do
    case('s4/menuclick/' .. spot[1], {
      reset = { mousemodel = 'popup' },
      setup = POPUP_SETUP,
      events = {
        { 'm', 'right', 'press', '', 2, 6 },
        { 'm', 'right', 'release', '', 2, 6 },
        -- The menu's own key loop must be running before the click
        -- that picks an entry, and nothing can be asked of the child
        -- while it is: `pum_show_popupmenu` serves no `nvim_exec_lua`.
        { 'w', 60 },
        { 'm', 'left', 'press', '', spot[2], spot[3] },
        { 'm', 'left', 'release', '', spot[2], spot[3] },
        { 'k', '\27' },
      },
      ans = { q = 'return vim.g.picked' },
    })
  end

  -- In Visual mode the menu shows the Visual-mode entries only.
  case('s4/visual/menu', {
    reset = { mousemodel = 'popup', mousetime = 100000 },
    setup = POPUP_SETUP,
    events = {
      { 'm', 'left', 'press', '', 0, 2 },
      { 'm', 'left', 'drag', '', 0, 9 },
      { 'm', 'left', 'release', '', 0, 9 },
      { 'm', 'right', 'press', '', 0, 5 },
      { 'm', 'right', 'release', '', 0, 5 },
      { 'w', 60 },
      { 'k', 'j' },
      { 'k', 'j' },
      { 'k', '\r' },
      { 'k', '\27' },
    },
    ans = { q = 'return vim.g.picked' },
  })

  -- No PopUp menu at all: the right click has nothing to show.
  case('s4/nomenu', {
    reset = { mousemodel = 'popup' },
    setup = "pcall(vim.cmd, 'silent! aunmenu PopUp') vim.g.picked = 'none' vim.cmd('redraw')",
    events = {
      { 'm', 'right', 'press', '', 2, 6 },
      { 'm', 'right', 'release', '', 2, 6 },
      { 'k', '\27' },
    },
    ans = { q = 'return {vim.g.picked, vim.v.errmsg}' },
  })

  -- `popup_setpos` moves the cursor to the click before showing; plain
  -- `popup` does not.  Read over a spread of positions.
  for _, mm in ipairs({ 'popup', 'popup_setpos' }) do
    for _, spot in ipairs({ { 0, 0 }, { 1, 9 }, { 4, 4 }, { 6, 0 }, { 8, 40 } }) do
      case(string.format('s4/setpos/%s/r%dc%d', mm, spot[1], spot[2]), {
        reset = { mousemodel = mm },
        setup = POPUP_SETUP,
        events = {
          { 'm', 'right', 'press', '', spot[1], spot[2] },
          { 'm', 'right', 'release', '', spot[1], spot[2] },
          { 'k', '\27' },
        },
        ans = { q = 'return vim.g.picked' },
      })
    end
  end

  SYNC = true
  ask('s4/optvalues', CUR:lua([[
    local out = {}
    for _, v in ipairs({'extend','popup','popup_setpos','mousemenu','x'}) do
      local ok, e = pcall(function() vim.o.mousemodel = v end)
      out[#out+1] = v .. '=' .. tostring(ok) .. '/' .. (ok and vim.o.mousemodel or tostring(e):gsub('.*: ', ''))
    end
    vim.o.mousemodel = 'extend'
    return out
  ]]))

  CUR:stop()
end)

-- ==================================================================== s5
-- The wheel.  `do_mousescroll` / `do_mousescroll_horiz` /
-- `ins_mousescroll` against every `'mousescroll'` spelling, with and
-- without `'wrap'`, in and out of Insert mode, and over a window that
-- is not the current one.
-- ====================================================================

local LONGLINES = {}
for i = 1, 200 do
  LONGLINES[i] = string.format('L%03d %s', i, string.rep('abcdefghij', 12))
end

section('s5-wheel', function()
  CUR = child_new()

  local SCROLLS = {
    'ver:3,hor:6',
    'ver:0,hor:0',
    'ver:1,hor:1',
    'ver:20,hor:20',
    'ver:1,hor:0',
    'ver:0,hor:12',
    'ver:100000,hor:100000',
  }
  for _, ms in ipairs(SCROLLS) do
    for _, dir in ipairs({ 'up', 'down', 'left', 'right' }) do
      for _, wrap in ipairs({ true, false }) do
        case(string.format('s5/scroll/%s/%s/wrap%s', ms:gsub('[:,]', ''), dir, tostring(wrap)), {
          reset = { mousescroll = ms, wrap = wrap, lines = LONGLINES },
          setup = 'vim.cmd("normal! 50G") vim.cmd("normal! 40|") vim.cmd("redraw")',
          events = { { 'm', 'wheel', dir, '', 5, 10 } },
        })
      end
    end
  end

  -- Modifiers: shift-wheel is a page, ctrl-wheel is nothing here.
  for _, m in ipairs({ 'S', 'C', 'A', 'SC' }) do
    for _, dir in ipairs({ 'up', 'down', 'left', 'right' }) do
      case(string.format('s5/mods/%s/%s', m, dir), {
        reset = { lines = LONGLINES, wrap = false },
        setup = 'vim.cmd("normal! 50G") vim.cmd("redraw")',
        events = { { 'm', 'wheel', dir, m, 5, 10 } },
      })
    end
  end

  -- At the very top and the very bottom, where the scroll is refused.
  for _, spot in ipairs({ { 'top', '1G' }, { 'bottom', 'G' } }) do
    for _, dir in ipairs({ 'up', 'down' }) do
      case('s5/edge/' .. spot[1] .. '/' .. dir, {
        reset = { lines = LONGLINES },
        setup = string.format('vim.cmd("normal! %s") vim.cmd("redraw")', spot[2]),
        events = { { 'm', 'wheel', dir, '', 5, 10 }, { 'm', 'wheel', dir, '', 5, 10 } },
      })
    end
  end

  -- `'scrolloff'` drags the cursor along with the view.
  for _, so in ipairs({ 0, 3, 999 }) do
    for _, dir in ipairs({ 'up', 'down' }) do
      case(string.format('s5/scrolloff/%d/%s', so, dir), {
        reset = { lines = LONGLINES, scrolloff = so },
        setup = 'vim.cmd("normal! 50G") vim.cmd("redraw")',
        events = { { 'm', 'wheel', dir, '', 5, 10 } },
      })
    end
  end

  -- In Insert mode, where `ins_mousescroll` runs instead.
  for _, dir in ipairs({ 'up', 'down', 'left', 'right' }) do
    case('s5/insert/' .. dir, {
      reset = { lines = LONGLINES, wrap = false },
      setup = 'vim.cmd("normal! 50G") vim.cmd("startinsert") vim.cmd("redraw")',
      want = 'i',
      events = { { 'm', 'wheel', dir, '', 5, 10 } },
    })
  end

  -- Over a window that is not the current one: the wheel scrolls the
  -- window under the pointer and leaves the cursor where it was.
  for _, dir in ipairs({ 'up', 'down' }) do
    for _, row in ipairs({ 2, 15 }) do
      case(string.format('s5/otherwin/%s/r%d', dir, row), {
        reset = { lines = LONGLINES },
        setup = 'vim.cmd("split") vim.cmd("normal! 50G") vim.cmd("wincmd j") vim.cmd("normal! 100G") vim.cmd("wincmd k") vim.cmd("redraw")',
        events = { { 'm', 'wheel', dir, '', row, 10 } },
        ans = {
          wins = true,
          q = 'local o = {} for _, w in ipairs(vim.api.nvim_list_wins()) do o[#o+1] = {w, vim.fn.line("w0", w), vim.fn.line("w$", w)} end return o',
        },
      })
    end
  end

  -- Horizontal wheel with `'wrap'` on is a no-op; with `'nowrap'` it
  -- moves `leftcol`, and `vcol2col` then decides the cursor column.
  for _, dir in ipairs({ 'left', 'right' }) do
    for _, sso in ipairs({ 0, 5 }) do
      case(string.format('s5/horiz/%s/sso%d', dir, sso), {
        reset = { lines = LONGLINES, wrap = false, sidescrolloff = sso },
        setup = 'vim.cmd("normal! 50G") vim.cmd("normal! 60|") vim.cmd("redraw")',
        events = { { 'm', 'wheel', dir, '', 5, 10 }, { 'm', 'wheel', dir, '', 5, 10 } },
      })
    end
  end

  -- `'smoothscroll'` puts the wheel into `skipcol` rather than
  -- `topline`.
  for _, dir in ipairs({ 'up', 'down' }) do
    case('s5/smooth/' .. dir, {
      reset = { lines = LONGLINES },
      setup = 'vim.wo.smoothscroll = true vim.cmd("normal! 50G") vim.cmd("redraw")',
      events = { { 'm', 'wheel', dir, '', 5, 10 } },
    })
  end

  CUR:stop()
end)

-- ==================================================================== s6
-- Drag-resize.  The answer is the window LAYOUT: dragging a status line
-- or a vertical separator is the one gesture whose whole effect is
-- geometry.  `'mouse'` gates this too.
-- ====================================================================

section('s6-drag', function()
  CUR = child_new()

  -- A horizontal split: the shared status line is at screen row 12,
  -- i.e. MOUSE row 11.
  for _, delta in ipairs({ -8, -3, -1, 0, 1, 3, 8, 20 }) do
    case('s6/hsplit/' .. delta, {
      setup = 'vim.cmd("split") vim.cmd("redraw")',
      events = {
        { 'm', 'left', 'press', '', 11, 5 },
        { 'm', 'left', 'drag', '', 11 + delta, 5 },
        { 'm', 'left', 'release', '', 11 + delta, 5 },
      },
      ans = { wins = true },
    })
  end

  -- A vertical split: the separator column of an 80-column screen split
  -- in two is column 40.
  for _, delta in ipairs({ -20, -5, -1, 0, 1, 5, 20, 60 }) do
    case('s6/vsplit/' .. delta, {
      setup = 'vim.cmd("vsplit") vim.cmd("redraw")',
      events = {
        { 'm', 'left', 'press', '', 5, 40 },
        { 'm', 'left', 'drag', '', 5, 40 + delta },
        { 'm', 'left', 'release', '', 5, 40 + delta },
      },
      ans = { wins = true },
    })
  end

  -- Which column actually grabs the separator.
  for _, col in ipairs({ 38, 39, 40, 41, 42 }) do
    case('s6/grab/col' .. col, {
      setup = 'vim.cmd("vsplit") vim.cmd("redraw")',
      events = {
        { 'm', 'left', 'press', '', 5, col },
        { 'm', 'left', 'drag', '', 5, col + 10 },
        { 'm', 'left', 'release', '', 5, col + 10 },
      },
      ans = { wins = true },
    })
  end
  for _, row in ipairs({ 10, 11, 12 }) do
    case('s6/grabrow/r' .. row, {
      setup = 'vim.cmd("split") vim.cmd("redraw")',
      events = {
        { 'm', 'left', 'press', '', row, 5 },
        { 'm', 'left', 'drag', '', row + 4, 5 },
        { 'm', 'left', 'release', '', row + 4, 5 },
      },
      ans = { wins = true },
    })
  end

  -- `'mouse'` and the resize gesture.
  for _, mv in ipairs({ '', 'n', 'v', 'a' }) do
    case('s6/mouse/' .. (mv == '' and 'empty' or mv), {
      reset = { mouse = mv },
      setup = 'vim.cmd("split") vim.cmd("redraw")',
      events = {
        { 'm', 'left', 'press', '', 11, 5 },
        { 'm', 'left', 'drag', '', 15, 5 },
        { 'm', 'left', 'release', '', 15, 5 },
      },
      ans = { wins = true },
    })
  end

  -- Three windows, dragging the middle status line, and a drag that
  -- would take a window below `'winminheight'`.
  case('s6/three/middle', {
    setup = 'vim.cmd("split") vim.cmd("split") vim.cmd("redraw")',
    events = {
      { 'm', 'left', 'press', '', 7, 5 },
      { 'm', 'left', 'drag', '', 3, 5 },
      { 'm', 'left', 'release', '', 3, 5 },
    },
    ans = { wins = true },
  })
  case('s6/three/collapse', {
    setup = 'vim.cmd("split") vim.cmd("split") vim.cmd("redraw")',
    events = {
      { 'm', 'left', 'press', '', 7, 5 },
      { 'm', 'left', 'drag', '', 0, 5 },
      { 'm', 'left', 'release', '', 0, 5 },
    },
    ans = { wins = true },
  })

  -- Dragging a tab label moves the tab page.
  for _, spot in ipairs({ { 'to2', 0, 3, 12 }, { 'to3', 0, 3, 22 }, { 'back', 0, 22, 3 } }) do
    case('s6/tabdrag/' .. spot[1], {
      reset = { showtabline = 2 },
      setup = 'vim.cmd("silent! tabnew") vim.cmd("silent! tabnew") vim.cmd("silent! tabfirst") vim.cmd("redraw")',
      events = {
        { 'm', 'left', 'press', '', spot[2], spot[3] },
        { 'm', 'left', 'drag', '', spot[2], spot[4] },
        { 'm', 'left', 'release', '', spot[2], spot[4] },
      },
      ans = { wins = true, rows = { 1 } },
    })
  end

  -- Middle click on the tab line closes a tab page.
  case('s6/tabclose/middle', {
    reset = { showtabline = 2 },
    setup = 'vim.cmd("silent! tabnew") vim.cmd("silent! tabnew") vim.cmd("silent! tabfirst") vim.cmd("redraw")',
    events = {
      { 'm', 'middle', 'press', '', 0, 12 },
      { 'm', 'middle', 'release', '', 0, 12 },
    },
    ans = { wins = true, rows = { 1 } },
  })

  -- A RIGHT drag that starts in the text and ENDS on a status line, a
  -- separator or a winbar.  This is the only place `jump_to_mouse`'s
  -- `on_status_line` is ever read: the drag-RESIZE path above goes
  -- through `status_line_offset`, which is computed from `below_window`
  -- and not from `on_status_line` at all.  Without these four cases a
  -- mutant that breaks `on_status_line` outright changes nothing this
  -- sweep can see.
  for _, target in ipairs({
    { 'onto-status', 22, 5, 'vim.cmd("vsplit")' },
    { 'onto-sep', 5, 40, 'vim.cmd("vsplit")' },
    { 'onto-winbar', 0, 5, 'vim.o.winbar = "WINBAR" vim.cmd("mode")' },
    { 'onto-text', 10, 5, 'vim.cmd("vsplit")' },
  }) do
    case('s6/rightdrag/' .. target[1], {
      reset = { mousemodel = 'extend' },
      -- With a Visual selection already up, which is the only state in
      -- which the right button does anything under `extend`.
      setup = target[4] .. ' vim.cmd("normal! 2G") vim.cmd("silent! normal! v2j4l") vim.cmd("redraw")',
      events = {
        { 'm', 'right', 'press', '', 3, 5 },
        { 'm', 'right', 'drag', '', target[2], target[3] },
        { 'm', 'right', 'release', '', target[2], target[3] },
      },
      ans = { wins = true },
    })
  end
  -- ... and the arm the four cases above do NOT reach.  B19-11 measured
  -- it: `on_status_line && which_button == MOUSE_RIGHT` needs a right
  -- PRESS on a status line while `status_line_offset` is still 0 from an
  -- earlier click in the text -- a right DRAG onto the status line goes
  -- through `status_line_offset` instead and never reads the static.
  -- The effect is only visible on a buffer TALLER than the window: with
  -- the arm broken the press falls through to the ordinary jump, which
  -- scrolls the view and starts Visual mode.  (Widening the four rows
  -- above was not the fix B19-11 thought it was -- `ANS` already reports
  -- `cur`/`vs`/`vw`/`md` for every case, so the missing thing was the
  -- GESTURE, not the answer.)
  local LONGBUF = 'local L = {} '
    .. 'for i = 1, 400 do L[i] = ("line %03d"):format(i) end '
    .. 'vim.api.nvim_buf_set_lines(0, 0, -1, false, L) '
    .. 'vim.api.nvim_win_set_cursor(0, {1, 0}) '
  for _, model in ipairs({ 'popup_setpos', 'popup', 'extend' }) do
    case('s6/rightpress/onto-status-' .. model, {
      reset = { mousemodel = model },
      setup = LONGBUF .. 'vim.cmd("redraw")',
      events = {
        { 'm', 'left', 'press', '', 3, 5 },
        { 'm', 'left', 'release', '', 3, 5 },
        { 'm', 'right', 'press', '', 22, 10 },
        { 'm', 'right', 'release', '', 22, 10 },
      },
      ans = { wins = true },
    })
  end
  -- (No `:split` variant: a case that opens a window shifts every later
  -- window id in the section, and the addition would stop being the pure
  -- `0 removed / N added` the manual asks a re-baseline to be.)

  -- ... and the same gesture with the LEFT button, which takes a
  -- different arm of the same chain.
  for _, target in ipairs({ { 'onto-status', 22, 5 }, { 'onto-sep', 5, 40 } }) do
    case('s6/leftdrag/' .. target[1], {
      reset = { mousemodel = 'extend' },
      setup = 'vim.cmd("vsplit") vim.cmd("redraw")',
      events = {
        { 'm', 'left', 'press', '', 3, 5 },
        { 'm', 'left', 'drag', '', target[2], target[3] },
        { 'm', 'left', 'release', '', target[2], target[3] },
      },
      ans = { wins = true },
    })
  end

  -- A drag that never saw a press: `did_drag` decides.
  case('s6/dragonly', {
    setup = 'vim.cmd("split") vim.cmd("redraw")',
    events = {
      { 'm', 'left', 'drag', '', 15, 5 },
      { 'm', 'left', 'release', '', 15, 5 },
    },
    ans = { wins = true },
  })

  CUR:stop()
end)

-- ==================================================================== s7
-- `%@Func@` click definitions, reached by a REAL click.  stlsweep s6
-- covers the arenas from the statusline side; this covers the dispatch
-- from the mouse side -- `call_click_def_func`, the button and modifier
-- strings it builds and the click COUNT it passes on.
-- ====================================================================

local CLICKFUNCS = [[
vim.cmd([==[
  func! MsClick(minwid, clicks, button, mods) abort
    call add(g:msc, printf('%d/%d/%s/[%s]', a:minwid, a:clicks, a:button, a:mods))
  endfunc
]==])
vim.g.msc = {}
]]

section('s7-clickdef', function()
  CUR = child_new()

  local TABLINE = '%1T%@MsClick@one%X%2T%@MsClick@two%X%99Xclose'
  local STATUS = 'lt%0@MsClick@CLICKY%X mid %5@MsClick@OTHER%X rt'
  local WINBAR = 'wb%7@MsClick@BAR%X end'

  local function clickcase(label, opts)
    case(label, {
      reset = opts.reset or { showtabline = 2 },
      setup = CLICKFUNCS .. opts.setup .. '\nvim.cmd("mode") vim.cmd("redraw")',
      events = opts.events,
      ans = { rows = opts.rows, q = 'return vim.g.msc' },
    })
  end

  -- Every button and every modifier through one tab-line region, so the
  -- string `call_click_def_func` builds is pinned character for
  -- character.
  for _, b in ipairs(BUTTONS) do
    clickcase('s7/button/' .. b, {
      setup = 'vim.o.tabline = ' .. string.format('%q', TABLINE),
      events = { { 'm', b, 'press', '', 0, 1 }, { 'm', b, 'release', '', 0, 1 } },
      rows = { 1 },
    })
  end
  for _, m in ipairs(MODS) do
    clickcase('s7/mods/' .. (m == '' and 'none' or m), {
      setup = 'vim.o.tabline = ' .. string.format('%q', TABLINE),
      events = { { 'm', 'left', 'press', m, 0, 1 }, { 'm', 'left', 'release', m, 0, 1 } },
      rows = { 1 },
    })
  end

  -- The click COUNT the handler is told about, pinned both ways.
  for _, mt in ipairs({ 0, 100000 }) do
    for n = 1, 4 do
      local ev = {}
      for _ = 1, n do
        ev[#ev + 1] = { 'm', 'left', 'press', '', 0, 1 }
        ev[#ev + 1] = { 'm', 'left', 'release', '', 0, 1 }
      end
      clickcase(string.format('s7/count/mt%d/%d', mt, n), {
        reset = { showtabline = 2, mousetime = mt },
        setup = 'vim.o.tabline = ' .. string.format('%q', TABLINE),
        events = ev,
        rows = { 1 },
      })
    end
  end

  -- Every column of the tab line, so the region boundaries are exact.
  for col = 0, 20 do
    clickcase(string.format('s7/col/%02d', col), {
      setup = 'vim.o.tabline = ' .. string.format('%q', TABLINE),
      events = { { 'm', 'left', 'press', '', 0, col }, { 'm', 'left', 'release', '', 0, col } },
      rows = { 1 },
    })
  end

  -- The status line is screen row 23 = MOUSE row 22, and the winbar is
  -- screen row 2 = MOUSE row 1 with `showtabline=2`.
  for _, col in ipairs({ 0, 1, 2, 7, 8, 12, 13, 18, 19, 30 }) do
    clickcase('s7/status/c' .. col, {
      setup = 'vim.o.statusline = ' .. string.format('%q', STATUS),
      events = { { 'm', 'left', 'press', '', 22, col }, { 'm', 'left', 'release', '', 22, col } },
      rows = { 23 },
    })
  end
  for _, col in ipairs({ 0, 1, 2, 4, 5, 8 }) do
    clickcase('s7/winbar/c' .. col, {
      setup = 'vim.o.winbar = ' .. string.format('%q', WINBAR),
      events = { { 'm', 'left', 'press', '', 1, col }, { 'm', 'left', 'release', '', 1, col } },
      rows = { 2 },
    })
  end

  -- `%NT` selects a tab page and `%NX` closes one.
  clickcase('s7/tabs/select', {
    setup = [[
      vim.cmd('silent! tabnew') vim.cmd('silent! tabnew') vim.cmd('silent! tabfirst')
      vim.o.tabline = '%1T[one]%2T[two]%3T[three]%T%=%999X[X]'
    ]],
    events = { { 'm', 'left', 'press', '', 0, 6 }, { 'm', 'left', 'release', '', 0, 6 } },
    rows = { 1 },
  })
  clickcase('s7/tabs/close', {
    setup = [[
      vim.cmd('silent! tabnew') vim.cmd('silent! tabnew') vim.cmd('silent! tabfirst')
      vim.o.tabline = '%1T[one]%2T[two]%3T[three]%T%=%999X[X]'
    ]],
    events = { { 'm', 'left', 'press', '', 0, 78 }, { 'm', 'left', 'release', '', 0, 78 } },
    rows = { 1 },
  })
  -- The default tab line, with no `'tabline'` set at all.
  for _, col in ipairs({ 0, 5, 14, 30 }) do
    clickcase('s7/deftab/c' .. col, {
      setup = [[
        vim.cmd('silent! tabnew') vim.cmd('silent! tabnew') vim.cmd('silent! tabfirst')
      ]],
      events = { { 'm', 'left', 'press', '', 0, col }, { 'm', 'left', 'release', '', 0, col } },
      rows = { 1 },
    })
  end

  -- A `'statuscolumn'` click definition, which lives in a third arena.
  for _, col in ipairs({ 0, 1, 2, 3 }) do
    clickcase('s7/statuscol/c' .. col, {
      setup = "vim.o.statuscolumn = '%@MsClick@%l%X '",
      events = { { 'm', 'left', 'press', '', 4, col }, { 'm', 'left', 'release', '', 4, col } },
      rows = { 5 },
    })
  end

  CUR:stop()
end)

-- ==================================================================== s8
-- The columns to the left of the text: fold, sign, number and
-- `'statuscolumn'`.  A click in the fold column opens or closes a fold;
-- a click anywhere else in the gutter is a click on the first text
-- column, and `jump_to_mouse` has to say so.
-- ====================================================================

section('s8-columns', function()
  CUR = child_new()

  local FOLDLINES = {}
  for i = 1, 30 do
    FOLDLINES[i] = string.format('line %02d %s', i, string.rep('.', i))
  end

  -- One closed fold, clicked at every fold-column width.
  for _, fc in ipairs({ '0', '1', '2', '4', '9', 'auto:9' }) do
    for _, col in ipairs({ 0, 1, 2, 3, 4, 8 }) do
      case(string.format('s8/fold/fc%s/c%d', fc:gsub(':', ''), col), {
        reset = { foldcolumn = fc, lines = FOLDLINES },
        setup = 'vim.cmd("3,8fold") vim.cmd("normal! 1G") vim.cmd("redraw")',
        events = {
          { 'm', 'left', 'press', '', 2, col },
          { 'm', 'left', 'release', '', 2, col },
        },
        ans = { q = 'return {vim.fn.foldclosed(3), vim.fn.foldclosed(5), vim.fn.foldlevel(5)}' },
      })
    end
  end

  -- Nested folds: the outer marker is column 0 and the inner column 1.
  for _, col in ipairs({ 0, 1, 2, 3 }) do
    case('s8/nested/c' .. col, {
      reset = { foldcolumn = '4', lines = FOLDLINES },
      -- `zo` must be run ON the fold: at line 1 it is `E490: No fold
    -- found` and the nesting never happens.
    setup = 'vim.cmd("2,20fold") vim.cmd("normal! 3Gzo") vim.cmd("5,10fold") vim.cmd("normal! 1G") vim.cmd("redraw")',
      events = {
        { 'm', 'left', 'press', '', 4, col },
        { 'm', 'left', 'release', '', 4, col },
      },
      ans = { q = 'return {vim.fn.foldclosed(2), vim.fn.foldclosed(5), vim.fn.foldlevel(6)}' },
    })
  end
  -- Clicking a CLOSED fold's text line, and a second click that reopens.
  case('s8/fold/reopen', {
    reset = { foldcolumn = '2', lines = FOLDLINES },
    setup = 'vim.cmd("3,8fold") vim.cmd("normal! 1G") vim.cmd("redraw")',
    events = {
      { 'm', 'left', 'press', '', 2, 0 },
      { 'm', 'left', 'release', '', 2, 0 },
      { 'm', 'left', 'press', '', 2, 0 },
      { 'm', 'left', 'release', '', 2, 0 },
    },
    ans = { q = 'return {vim.fn.foldclosed(3)}' },
  })
  case('s8/fold/textclick', {
    reset = { foldcolumn = '0', lines = FOLDLINES },
    setup = 'vim.cmd("3,8fold") vim.cmd("normal! 1G") vim.cmd("redraw")',
    events = {
      { 'm', 'left', 'press', '', 2, 10 },
      { 'm', 'left', 'release', '', 2, 10 },
    },
    ans = { q = 'return {vim.fn.foldclosed(3)}' },
  })

  -- The sign column.
  for _, sc in ipairs({ 'auto', 'yes', 'no', 'yes:2', 'number' }) do
    for _, col in ipairs({ 0, 1, 2, 3, 6 }) do
      case(string.format('s8/sign/%s/c%d', sc:gsub(':', ''), col), {
        reset = { signcolumn = sc, lines = FOLDLINES },
        setup = [[
          vim.fn.sign_define('MsSign', {text='>>'})
          vim.fn.sign_place(0, '', 'MsSign', vim.api.nvim_get_current_buf(), {lnum=3})
          vim.cmd('redraw')
        ]],
        events = {
          { 'm', 'left', 'press', '', 2, col },
          { 'm', 'left', 'release', '', 2, col },
        },
      })
    end
  end

  -- The number column, absolute and relative.
  for _, nu in ipairs({ 'number', 'relativenumber', 'both' }) do
    for _, col in ipairs({ 0, 1, 2, 3, 4, 5 }) do
      case(string.format('s8/number/%s/c%d', nu, col), {
        reset = {
          lines = FOLDLINES,
          number = (nu ~= 'relativenumber'),
          relativenumber = (nu ~= 'number'),
        },
        setup = 'vim.cmd("normal! 5G") vim.cmd("redraw")',
        events = {
          { 'm', 'left', 'press', '', 9, col },
          { 'm', 'left', 'release', '', 9, col },
        },
      })
    end
  end

  -- Everything at once, so the offsets add up.
  for _, col in ipairs({ 0, 2, 4, 6, 8, 10, 12 }) do
    case('s8/all/c' .. col, {
      reset = { lines = FOLDLINES, number = true, foldcolumn = '3', signcolumn = 'yes' },
      setup = [[
        vim.fn.sign_define('MsSign', {text='>>'})
        vim.fn.sign_place(0, '', 'MsSign', vim.api.nvim_get_current_buf(), {lnum=4})
        vim.cmd('redraw')
      ]],
      events = {
        { 'm', 'left', 'press', '', 3, col },
        { 'm', 'left', 'release', '', 3, col },
      },
    })
  end

  -- A `'statuscolumn'` of a fixed width.
  for _, col in ipairs({ 0, 1, 2, 3, 4, 5, 6 }) do
    case('s8/statuscol/c' .. col, {
      reset = { lines = FOLDLINES },
      setup = "vim.o.statuscolumn = '[%l]' vim.cmd('redraw')",
      events = {
        { 'm', 'left', 'press', '', 3, col },
        { 'm', 'left', 'release', '', 3, col },
      },
    })
  end

  CUR:stop()
end)

-- ==================================================================== s9
-- The modes.  `ins_mouse` in Insert, the Visual extend arms, Select
-- mode, an operator waiting for a motion, and the command line.
-- ====================================================================

section('s9-modes', function()
  CUR = child_new()

  -- Insert mode: a click, a drag, and a click past the end of a line.
  for _, spot in ipairs({
    { 'mid', 1, 6 },
    { 'sol', 1, 0 },
    { 'past-eol', 5, 40 },
    { 'emptyline', 6, 10 },
    { 'below-last', 12, 4 },
    { 'onstatus', 22, 4 },
  }) do
    case('s9/insert/' .. spot[1], {
      setup = 'vim.cmd("normal! 1G") vim.cmd("startinsert") vim.cmd("redraw")',
      want = 'i',
      events = {
        { 'm', 'left', 'press', '', spot[2], spot[3] },
        { 'm', 'left', 'release', '', spot[2], spot[3] },
      },
    })
  end
  case('s9/insert/drag', {
    setup = 'vim.cmd("normal! 1G") vim.cmd("startinsert") vim.cmd("redraw")',
    want = 'i',
    events = {
      { 'm', 'left', 'press', '', 0, 2 },
      { 'm', 'left', 'drag', '', 1, 8 },
      { 'm', 'left', 'release', '', 1, 8 },
    },
  })
  -- `'virtualedit'` decides whether Insert can sit past the end.
  for _, ve in ipairs({ '', 'all', 'onemore', 'block' }) do
    case('s9/ve/' .. (ve == '' and 'none' or ve), {
      reset = { virtualedit = ve },
      setup = 'vim.cmd("normal! 6G") vim.cmd("redraw")',
      events = {
        { 'm', 'left', 'press', '', 5, 40 },
        { 'm', 'left', 'release', '', 5, 40 },
      },
    })
  end

  -- Visual mode: a click inside a selection, outside it, and with
  -- Shift, in each of the three Visual kinds.
  for _, kind in ipairs({ { 'char', 'v' }, { 'line', 'V' }, { 'block', '\22' } }) do
    for _, spot in ipairs({ { 'inside', 1, 6 }, { 'before', 0, 1 }, { 'after', 3, 8 } }) do
      for _, m in ipairs({ '', 'S' }) do
        case(string.format('s9/visual/%s/%s/%s', kind[1], spot[1], m == '' and 'plain' or 'shift'), {
          setup = string.format(
            'vim.cmd("normal! 1G") vim.cmd("silent! normal! %s2j5l") vim.cmd("redraw")',
            kind[2]
          ),
          events = {
            { 'm', 'left', 'press', m, spot[2], spot[3] },
            { 'm', 'left', 'release', m, spot[2], spot[3] },
          },
        })
      end
    end
  end

  -- Select mode, where a click ends the selection.
  case('s9/select/click', {
    reset = { selectmode = 'mouse' },
    setup = 'vim.cmd("normal! 1G") vim.cmd("redraw")',
    events = {
      { 'm', 'left', 'press', '', 0, 2 },
      { 'm', 'left', 'drag', '', 1, 8 },
      { 'm', 'left', 'release', '', 1, 8 },
    },
  })
  case('s9/select/type', {
    reset = { selectmode = 'mouse' },
    setup = 'vim.cmd("normal! 1G") vim.cmd("redraw")',
    events = {
      { 'm', 'left', 'press', '', 0, 2 },
      { 'm', 'left', 'drag', '', 0, 8 },
      { 'm', 'left', 'release', '', 0, 8 },
      { 'k', 'Z' },
    },
    ans = { q = 'return vim.api.nvim_buf_get_lines(0, 0, 2, false)' },
  })

  -- An operator waiting for a motion: the click IS the motion.
  for _, op in ipairs({ 'd', 'y', 'c', 'g~' }) do
    case('s9/operator/' .. op:gsub('~', 'tilde'), {
      setup = 'vim.cmd("normal! 1G") vim.cmd("redraw")',
      events = {
        { 'k', op },
        { 'm', 'left', 'press', '', 1, 8 },
        { 'm', 'left', 'release', '', 1, 8 },
        { 'k', '\27' },
      },
      ans = { q = 'return {vim.api.nvim_buf_get_lines(0, 0, 3, false), vim.fn.getreg(\'"\')}' },
    })
  end

  -- The command line.  `'mouse'` must contain `c`, and the click moves
  -- the cursor within the typed text.
  for _, col in ipairs({ 0, 1, 3, 5, 9, 20 }) do
    case('s9/cmdline/c' .. col, {
      events = {
        { 'k', ':abcdefghij' },
        { 'm', 'left', 'press', '', 23, col },
        { 'm', 'left', 'release', '', 23, col },
      },
      ans = { q = 'return {vim.fn.mode(), vim.fn.getcmdline(), vim.fn.getcmdpos()}' },
    })
  end
  case('s9/cmdline/offline', {
    events = {
      { 'k', ':abcdefghij' },
      { 'm', 'left', 'press', '', 10, 4 },
      { 'm', 'left', 'release', '', 10, 4 },
    },
    ans = { q = 'return {vim.fn.mode(), vim.fn.getcmdline(), vim.fn.getcmdpos()}' },
  })
  case('s9/cmdline/wheel', {
    events = {
      { 'k', ':abcdefghij' },
      { 'm', 'wheel', 'down', '', 23, 4 },
    },
    ans = { q = 'return {vim.fn.mode(), vim.fn.getcmdline(), vim.fn.getcmdpos()}' },
  })

  -- A terminal buffer.  `nvim_open_term` gives a real one with no job
  -- behind it, which is the only reproducible spelling (B19-2).
  --
  -- The `vim.wait` is load-bearing: a terminal buffer's lines are filled
  -- in behind `terminal/refresh.rs`'s 10 ms timer, so `nvim_chan_send`
  -- followed by `redraw` is a RACE -- the click can land on a line that
  -- is still empty and clamp to column 0.  It lost reliably-slowly until
  -- p22's S16 made debug builds ~2x faster (the GlobalCell borrow table),
  -- at which point this row started flipping about one run in three.
  -- Waiting for the text makes it deterministic AT THE BASELINE VALUE;
  -- nothing about the mouse moved and the baseline is unchanged.
  case('s9/terminal/click', {
    setup = [[
      local b = vim.api.nvim_create_buf(false, true)
      local t = vim.api.nvim_open_term(b, {})
      vim.api.nvim_chan_send(t, 'termline one\r\ntermline two\r\ntermline three\r\n')
      vim.api.nvim_win_set_buf(0, b)
      vim.wait(5000, function()
        local l = vim.api.nvim_buf_get_lines(b, 1, 2, false)[1]
        return l ~= nil and l:find('termline two') ~= nil
      end)
      vim.cmd('redraw')
    ]],
    events = {
      { 'm', 'left', 'press', '', 1, 5 },
      { 'm', 'left', 'release', '', 1, 5 },
    },
    ans = { q = 'return {vim.bo.buftype, vim.fn.mode()}' },
  })

  CUR:stop()
end)

-- =================================================================== s10
-- Floating windows.  `mouse_find_win_outer` walks the layer list from
-- the top down, and `focusable = false` and `mouse = false` each take a
-- float out of that walk in a different way.
-- ====================================================================

section('s10-floats', function()
  CUR = child_new()

  local FLOAT = [[
    local b = vim.api.nvim_create_buf(false, true)
    vim.api.nvim_buf_set_lines(b, 0, -1, false, {'FLOATLINE-ONE','FLOATLINE-TWO','FLOATLINE-3'})
    vim.g.fw = vim.api.nvim_open_win(b, false, vim.tbl_extend('force',
      {relative='editor', row=3, col=10, width=14, height=3, style='minimal'}, CFG))
    vim.cmd('redraw')
  ]]

  local VARIANTS = {
    { 'plain', '{}' },
    { 'nofocus', '{focusable=false}' },
    { 'nomouse', '{mouse=false}' },
    { 'border', "{border='single'}" },
    { 'zindex-low', '{zindex=10}' },
    { 'zindex-high', '{zindex=200}' },
  }
  for _, v in ipairs(VARIANTS) do
    for _, spot in ipairs({
      { 'inside', 4, 12 },
      { 'topleft', 3, 10 },
      { 'bottomright', 5, 23 },
      { 'border-top', 2, 12 },
      { 'border-left', 4, 9 },
      { 'outside', 8, 40 },
    }) do
      case(string.format('s10/%s/%s', v[1], spot[1]), {
        setup = 'CFG = ' .. v[2] .. '\n' .. FLOAT,
        events = {
          { 'm', 'left', 'press', '', spot[2], spot[3] },
          { 'm', 'left', 'release', '', spot[2], spot[3] },
        },
        ans = { wins = true },
      })
    end
  end

  -- Two overlapping floats: the higher zindex wins.
  for _, spot in ipairs({ { 'overlap', 4, 14 }, { 'onlyA', 4, 11 }, { 'onlyB', 4, 22 } }) do
    case('s10/overlap/' .. spot[1], {
      setup = [[
        local a = vim.api.nvim_create_buf(false, true)
        vim.api.nvim_buf_set_lines(a, 0, -1, false, {'AAAAAAAAAA'})
        local b = vim.api.nvim_create_buf(false, true)
        vim.api.nvim_buf_set_lines(b, 0, -1, false, {'BBBBBBBBBB'})
        vim.g.wa = vim.api.nvim_open_win(a, false, {relative='editor', row=3, col=10, width=8, height=3, style='minimal', zindex=50})
        vim.g.wb = vim.api.nvim_open_win(b, false, {relative='editor', row=3, col=14, width=8, height=3, style='minimal', zindex=60})
        vim.cmd('redraw')
      ]],
      events = {
        { 'm', 'left', 'press', '', spot[2], spot[3] },
        { 'm', 'left', 'release', '', spot[2], spot[3] },
      },
      ans = { wins = true, q = 'return {vim.g.wa, vim.g.wb}' },
    })
  end

  -- The wheel over a float scrolls the float, not the window below it.
  case('s10/wheel/onfloat', {
    setup = [[
      local b = vim.api.nvim_create_buf(false, true)
      local l = {} for i = 1, 100 do l[i] = 'F' .. i end
      vim.api.nvim_buf_set_lines(b, 0, -1, false, l)
      vim.g.fw = vim.api.nvim_open_win(b, false, {relative='editor', row=3, col=10, width=14, height=5, style='minimal'})
      vim.cmd('redraw')
    ]],
    events = { { 'm', 'wheel', 'down', '', 4, 12 } },
    ans = {
      wins = true,
      q = 'local o = {} for _, w in ipairs(vim.api.nvim_list_wins()) do o[#o+1] = {w, vim.fn.line("w0", w)} end return o',
    },
  })

  -- Dragging a float's border does not resize it; dragging the split
  -- below a float still finds the split.
  case('s10/drag/through', {
    setup = 'CFG = {focusable=false}\n' .. FLOAT .. '\nvim.cmd("split") vim.cmd("redraw")',
    events = {
      { 'm', 'left', 'press', '', 11, 12 },
      { 'm', 'left', 'drag', '', 14, 12 },
      { 'm', 'left', 'release', '', 14, 12 },
    },
    ans = { wins = true },
  })

  -- A float with its own status line, and one that covers the real one.
  case('s10/cover/status', {
    setup = [[
      local b = vim.api.nvim_create_buf(false, true)
      vim.api.nvim_buf_set_lines(b, 0, -1, false, {'COVER'})
      vim.g.fw = vim.api.nvim_open_win(b, false, {relative='editor', row=22, col=0, width=40, height=2, style='minimal'})
      vim.cmd('redraw')
    ]],
    events = {
      { 'm', 'left', 'press', '', 22, 5 },
      { 'm', 'left', 'release', '', 22, 5 },
    },
    ans = { wins = true, rows = { 23 } },
  })

  CUR:stop()
end)

-- =================================================================== s11
-- `getmousepos()` and the cursor column, over a grid of positions and
-- every column-affecting option.  This is `mouse_comp_pos` and
-- `vcol2col`, the arithmetic half of the file, and it is the section
-- with the highest row count for that reason.
-- ====================================================================

section('s11-mousepos', function()
  CUR = child_new()

  local GRIDLINES = {
    'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ',
    '\tone\ttwo\tthree',
    '\u{03b1}\u{03b2}\u{03b3}\u{03b4}\u{03b5}\u{03b6}\u{03b7}\u{03b8}\u{03b9}\u{03ba}',
    '\u{65e5}\u{672c}\u{8a9e}\u{30c6}\u{30ad}\u{30b9}\u{30c8}',
    'short',
    '',
  }

  local CONFIGS = {
    { 'plain', {} },
    { 'number', { number = true } },
    { 'foldcol', { foldcolumn = '4' } },
    { 'sign', { signcolumn = 'yes' } },
    { 'nowrap', { wrap = false } },
    { 've-all', { virtualedit = 'all' } },
  }
  for _, cfg in ipairs(CONFIGS) do
    local reset = vim.deepcopy(cfg[2])
    reset.lines = GRIDLINES
    for row = 0, 5 do
      for _, col in ipairs({ 0, 1, 2, 3, 4, 6, 9, 13, 20, 45, 79 }) do
        case(string.format('s11/%s/r%dc%02d', cfg[1], row, col), {
          reset = reset,
          events = {
            { 'm', 'left', 'press', '', row, col },
            { 'm', 'left', 'release', '', row, col },
          },
        })
      end
    end
  end

  -- With a horizontal offset, so `vcol2col` has real work to do.
  for _, lc in ipairs({ 0, 10, 40 }) do
    for _, col in ipairs({ 0, 5, 20, 60, 79 }) do
      case(string.format('s11/leftcol/%d/c%02d', lc, col), {
        reset = { lines = GRIDLINES, wrap = false },
        setup = string.format('vim.fn.winrestview({leftcol=%d}) vim.cmd("redraw")', lc),
        events = {
          { 'm', 'left', 'press', '', 0, col },
          { 'm', 'left', 'release', '', 0, col },
        },
      })
    end
  end

  -- A wrapped long line, where one buffer line owns several screen rows.
  for row = 0, 5 do
    for _, col in ipairs({ 0, 20, 60, 79 }) do
      case(string.format('s11/wrapped/r%dc%02d', row, col), {
        reset = { lines = { string.rep('0123456789', 30), 'after' } },
        events = {
          { 'm', 'left', 'press', '', row, col },
          { 'm', 'left', 'release', '', row, col },
        },
      })
    end
  end

  -- In a split, where the window offsets are non-zero.
  for _, spot in ipairs({ { 0, 0 }, { 5, 20 }, { 11, 5 }, { 12, 5 }, { 13, 5 }, { 20, 60 } }) do
    case(string.format('s11/split/r%dc%d', spot[1], spot[2]), {
      reset = { lines = GRIDLINES },
      setup = 'vim.cmd("split") vim.cmd("redraw")',
      events = {
        { 'm', 'left', 'press', '', spot[1], spot[2] },
        { 'm', 'left', 'release', '', spot[1], spot[2] },
      },
      ans = { wins = true },
    })
  end
  for _, spot in ipairs({ { 5, 0 }, { 5, 39 }, { 5, 40 }, { 5, 41 }, { 5, 79 } }) do
    case(string.format('s11/vsplit/r%dc%d', spot[1], spot[2]), {
      reset = { lines = GRIDLINES },
      setup = 'vim.cmd("vsplit") vim.cmd("redraw")',
      events = {
        { 'm', 'left', 'press', '', spot[1], spot[2] },
        { 'm', 'left', 'release', '', spot[1], spot[2] },
      },
      ans = { wins = true },
    })
  end

  CUR:stop()
end)

-- =================================================================== s12
-- `ext_multigrid`.  `mouse_check_grid` and `mouse_find_grid_win` only
-- do anything when a UI has asked for per-window grids, and no headless
-- bench or sweep has ever turned that on.
--
-- The UI client is a RAW socket channel (`rpc = false`): fourteen
-- hand-encoded msgpack bytes for
-- `[0, 1, "nvim_ui_attach", [80, 24, {"ext_multigrid": true}]]`, and
-- everything the child sends back is discarded.  An RPC channel cannot
-- be a UI, because the child's first `redraw` NOTIFICATION resolves to
-- no API handler in this parent and the channel is closed (B19-1).
-- ====================================================================

section('s12-multigrid', function()
  local sock = work .. '/ui.sock'
  os.remove(sock)
  CUR = child_new({ '--listen', sock })

  local function u8(...)
    return string.char(...)
  end
  local function fixstr(s)
    return u8(0xa0 + #s) .. s
  end
  local attach = u8(0x94)
    .. u8(0x00)
    .. u8(0x01)
    .. fixstr('nvim_ui_attach')
    .. u8(0x93)
    .. u8(0x50)
    .. u8(0x18)
    .. u8(0x81)
    .. fixstr('ext_multigrid')
    .. u8(0xc3)

  local uichan = -1
  local ok = pcall(function()
    uichan = vim.fn.sockconnect('pipe', sock, { rpc = false, on_data = function() end })
  end)
  if not ok or uichan <= 0 then
    ask('s12/ui', 'SOCKCONNECT FAILED')
    CUR:stop()
    return
  end
  vim.fn.chansend(uichan, attach)
  -- The attach is asynchronous; the first query that answers proves it
  -- landed.  A fixed wait would be a race on a loaded machine.
  local uis = nil
  for _ = 1, 100 do
    uis = CUR:lua('return {n = #vim.api.nvim_list_uis(), mg = vim.fn.has("nvim") == 1 and (vim.api.nvim_list_uis()[1] or {}).ext_multigrid}')
    if type(uis) == 'table' and uis.n and uis.n > 0 then
      break
    end
    vim.wait(20)
  end
  ask('s12/ui/attached', uis)

  -- Every grid id against a two-window layout.  Grid 1 is the outer
  -- grid; each window owns one of the rest.
  for _, g in ipairs({ 0, 1, 2, 3, 4, 5, 6, 99 }) do
    for _, spot in ipairs({ { 0, 0 }, { 1, 4 }, { 5, 12 } }) do
      case(string.format('s12/grid%d/r%dc%d', g, spot[1], spot[2]), {
        setup = 'vim.cmd("split") vim.cmd("wincmd j") vim.cmd("redraw")',
        events = {
          { 'm', 'left', 'press', '', spot[1], spot[2], g },
          { 'm', 'left', 'release', '', spot[1], spot[2], g },
        },
        ans = { wins = true },
      })
    end
  end

  -- A float has a grid of its own.
  for _, g in ipairs({ 0, 1, 2, 3, 4 }) do
    case('s12/float/grid' .. g, {
      setup = [[
        local b = vim.api.nvim_create_buf(false, true)
        vim.api.nvim_buf_set_lines(b, 0, -1, false, {'FLOATA','FLOATB','FLOATC'})
        vim.g.fw = vim.api.nvim_open_win(b, false, {relative='editor', row=3, col=10, width=12, height=3, style='minimal'})
        vim.cmd('redraw')
      ]],
      events = {
        { 'm', 'left', 'press', '', 1, 3, g },
        { 'm', 'left', 'release', '', 1, 3, g },
      },
      ans = { wins = true, q = 'return vim.g.fw' },
    })
  end

  -- The wheel and a drag-resize in grid coordinates.
  for _, g in ipairs({ 0, 1, 4 }) do
    case('s12/wheel/grid' .. g, {
      reset = { lines = LONGLINES },
      setup = 'vim.cmd("split") vim.cmd("normal! 50G") vim.cmd("redraw")',
      events = { { 'm', 'wheel', 'down', '', 2, 4, g } },
      ans = {
        wins = true,
        q = 'local o = {} for _, w in ipairs(vim.api.nvim_list_wins()) do o[#o+1] = {w, vim.fn.line("w0", w)} end return o',
      },
    })
  end

  pcall(vim.fn.chanclose, uichan)
  CUR:stop()
end)

-- =================================================================== s13
-- The gestures whose whole effect is somewhere other than the cursor:
-- the middle-button paste, the CTRL-click tag jump, the auto-scroll a
-- drag past the edge of a window starts, the window focus a click
-- moves, `'mousemoveevent'`, and the `extend` model's right click.
-- ====================================================================

section('s13-gestures', function()
  CUR = child_new()

  -- Middle click pastes.  The answer is the BUFFER, which is the only
  -- place this arm of `do_mouse` leaves a mark.
  for _, spot in ipairs({
    { 'sol', 0, 0 },
    { 'mid', 0, 6 },
    { 'eol', 0, 21 },
    { 'past-eol', 5, 40 },
    { 'empty', 6, 0 },
    { 'below-last', 12, 4 },
  }) do
    for _, reg in ipairs({ 'charwise', 'linewise', 'blockwise' }) do
      local set = ({
        charwise = 'vim.fn.setreg(\'"\', "PASTED", "v")',
        linewise = 'vim.fn.setreg(\'"\', "PASTEDLINE", "V")',
        blockwise = 'vim.fn.setreg(\'"\', {"P1", "P2"}, "b")',
      })[reg]
      case(string.format('s13/paste/%s/%s', reg, spot[1]), {
        setup = set .. ' vim.cmd("redraw")',
        events = {
          { 'm', 'middle', 'press', '', spot[2], spot[3] },
          { 'm', 'middle', 'release', '', spot[2], spot[3] },
        },
        ans = { q = 'return vim.api.nvim_buf_get_lines(0, 0, 8, false)' },
      })
    end
  end
  -- ... and in Insert mode, where it goes through `ins_mouse` instead.
  case('s13/paste/insert', {
    setup = 'vim.fn.setreg(\'"\', "PASTED", "v") vim.cmd("startinsert") vim.cmd("redraw")',
    want = 'i',
    events = {
      { 'm', 'middle', 'press', '', 1, 6 },
      { 'm', 'middle', 'release', '', 1, 6 },
    },
    ans = { q = 'return vim.api.nvim_buf_get_lines(0, 0, 3, false)' },
  })
  -- `'mouse'` empty must not change that.
  case('s13/paste/nomouse', {
    reset = { mouse = '' },
    setup = 'vim.fn.setreg(\'"\', "PASTED", "v") vim.cmd("redraw")',
    events = {
      { 'm', 'middle', 'press', '', 1, 6 },
      { 'm', 'middle', 'release', '', 1, 6 },
    },
    ans = { q = 'return vim.api.nvim_buf_get_lines(0, 0, 3, false)' },
  })

  -- CTRL-left is a tag jump and CTRL-right is CTRL-T.  With no tags
  -- file both answer an error, and the error IS the observable.
  for _, spot in ipairs({ { 'word', 0, 2 }, { 'space', 0, 5 }, { 'empty', 6, 0 } }) do
    for _, b in ipairs({ 'left', 'right' }) do
      case(string.format('s13/tagjump/%s/%s', b, spot[1]), {
        setup = 'vim.v.errmsg = "" vim.cmd("redraw")',
        events = {
          { 'm', b, 'press', 'C', spot[2], spot[3] },
          { 'm', b, 'release', 'C', spot[2], spot[3] },
        },
        ans = { q = 'return {vim.v.errmsg, #vim.fn.gettagstack().items}' },
      })
    end
  end

  -- A drag that leaves the window scrolls it.  This is the
  -- `jump_to_mouse` arm nothing else here reaches.
  for _, target in ipairs({
    { 'above', 0 },
    { 'wayabove', 0 },
    { 'below', 21 },
    { 'waybelow', 23 },
  }) do
    for _, so in ipairs({ 0, 5 }) do
      case(string.format('s13/dragscroll/%s/so%d', target[1], so), {
        reset = { lines = LONGLINES, scrolloff = so },
        setup = 'vim.cmd("normal! 50G") vim.cmd("normal! zz") vim.cmd("redraw")',
        events = {
          { 'm', 'left', 'press', '', 10, 5 },
          { 'm', 'left', 'drag', '', target[2], 5 },
          { 'm', 'left', 'drag', '', target[2], 5 },
          { 'm', 'left', 'release', '', target[2], 5 },
        },
      })
    end
  end

  -- Which window a click gives the focus to, and what the other one
  -- keeps.
  for _, layout in ipairs({ { 'split', 'split' }, { 'vsplit', 'vsplit' } }) do
    for _, spot in ipairs({ { 0, 0 }, { 5, 20 }, { 11, 5 }, { 13, 5 }, { 20, 5 }, { 5, 60 } }) do
      case(string.format('s13/focus/%s/r%dc%d', layout[1], spot[1], spot[2]), {
        reset = { lines = LONGLINES },
        setup = string.format(
          'vim.cmd("%s") vim.cmd("normal! 20G") vim.cmd("wincmd p") vim.cmd("normal! 60G") vim.cmd("redraw")',
          layout[2]
        ),
        events = {
          { 'm', 'left', 'press', '', spot[1], spot[2] },
          { 'm', 'left', 'release', '', spot[1], spot[2] },
        },
        ans = {
          wins = true,
          q = 'local o = {} for _, w in ipairs(vim.api.nvim_list_wins()) do o[#o+1] = {w, vim.api.nvim_win_get_cursor(w)[1]} end return o',
        },
      })
    end
  end

  -- `'mousemoveevent'`: a move is only delivered when it is asked for.
  for _, on in ipairs({ true, false }) do
    for _, spot in ipairs({ { 2, 6 }, { 7, 30 } }) do
      case(string.format('s13/move/%s/r%dc%d', tostring(on), spot[1], spot[2]), {
        setup = string.format('vim.o.mousemoveevent = %s vim.cmd("redraw")', tostring(on)),
        events = {
          { 'm', 'move', 'move', '', spot[1], spot[2] },
        },
        ans = { q = 'return vim.o.mousemoveevent' },
      })
    end
  end
  -- A move DURING a drag, which is a different key again.
  case('s13/move/duringdrag', {
    setup = 'vim.o.mousemoveevent = true vim.cmd("redraw")',
    events = {
      { 'm', 'left', 'press', '', 0, 2 },
      { 'm', 'move', 'move', '', 2, 10 },
      { 'm', 'left', 'drag', '', 2, 10 },
      { 'm', 'left', 'release', '', 2, 10 },
    },
  })

  -- The `extend` model's right click extends the Visual selection from
  -- whichever end is nearer.
  for _, spot in ipairs({ { 'before', 0, 0 }, { 'inside', 1, 4 }, { 'after', 3, 8 }, { 'sameline', 1, 20 } }) do
    case('s13/extend/' .. spot[1], {
      reset = { mousemodel = 'extend', mousetime = 0 },
      setup = 'vim.cmd("normal! 2G") vim.cmd("silent! normal! v5l") vim.cmd("redraw")',
      events = {
        { 'm', 'right', 'press', '', spot[2], spot[3] },
        { 'm', 'right', 'release', '', spot[2], spot[3] },
      },
    })
  end

  -- A `'winbar'` shifts every row of the window down by one, and a
  -- click has to account for it.
  for _, row in ipairs({ 0, 1, 2, 3 }) do
    case('s13/winbar/r' .. row, {
      setup = 'vim.o.winbar = "WINBAR" vim.cmd("mode") vim.cmd("redraw")',
      events = {
        { 'm', 'left', 'press', '', row, 4 },
        { 'm', 'left', 'release', '', row, 4 },
      },
      ans = { rows = { 1, 2 } },
    })
  end

  CUR:stop()
end)

-- =================================================================== s91
-- crashprobe.  The inputs that might kill the editor, one CHILD each,
-- so that a death is one diffable ABORTED row rather than a truncated
-- report.  The verdict is whether the child still answers afterwards.
-- ====================================================================

section('s91-crashprobe', function()
  local PROBES = {
    { 'huge-row', { { 'm', 'left', 'press', '', 2147483647, 4 }, { 'm', 'left', 'release', '', 2147483647, 4 } } },
    { 'huge-col', { { 'm', 'left', 'press', '', 2, 2147483647 }, { 'm', 'left', 'release', '', 2, 2147483647 } } },
    { 'neg-row', { { 'm', 'left', 'press', '', -1, 4 } } },
    { 'neg-col', { { 'm', 'left', 'press', '', 2, -1 } } },
    { 'huge-grid', { { 'm', 'left', 'press', '', 2, 4, 2147483647 } } },
    { 'neg-grid', { { 'm', 'left', 'press', '', 2, 4, -1 } } },
    { 'grid-noui', { { 'm', 'left', 'press', '', 2, 4, 7 }, { 'm', 'left', 'release', '', 2, 4, 7 } } },
    { 'bad-button', { { 'm', 'nosuch', 'press', '', 2, 4 } } },
    { 'bad-action', { { 'm', 'left', 'nosuch', '', 2, 4 } } },
    { 'bad-mods', { { 'm', 'left', 'press', 'QQQ', 2, 4 } } },
    { 'wheel-press', { { 'm', 'wheel', 'press', '', 2, 4 } } },
    { 'left-up', { { 'm', 'left', 'up', '', 2, 4 } } },
    { 'move-noevent', { { 'm', 'move', 'move', '', 2, 4 } } },
    { 'drag-noclick', { { 'm', 'left', 'drag', '', 5, 5 }, { 'm', 'left', 'drag', '', 6, 6 } } },
    { 'release-noclick', { { 'm', 'left', 'release', '', 5, 5 } } },
    { 'statusdrag-noclick', { { 'm', 'left', 'drag', '', 22, 5 }, { 'm', 'left', 'release', '', 22, 5 } } },
    { 'cmdline-drag', { { 'k', ':x' }, { 'm', 'left', 'drag', '', 23, 1 }, { 'k', '\27' } } },
    {
      'hundred-clicks',
      (function()
        local ev = {}
        for _ = 1, 100 do
          ev[#ev + 1] = { 'm', 'left', 'press', '', 1, 5 }
          ev[#ev + 1] = { 'm', 'left', 'release', '', 1, 5 }
        end
        return ev
      end)(),
      { mousetime = 100000 },
    },
    { 'drag-off-top', { { 'm', 'left', 'press', '', 5, 5 }, { 'm', 'left', 'drag', '', 0, 0 }, { 'm', 'left', 'drag', '', 23, 79 } } },
    { 'wheel-empty', { { 'm', 'wheel', 'down', '', 2, 4 } }, { lines = { '' } } },
    { 'wheel-zero', { { 'm', 'wheel', 'down', '', 2, 4 }, { 'm', 'wheel', 'right', '', 2, 4 } }, { mousescroll = 'ver:0,hor:0' } },
    { 'fold-lastline', { { 'm', 'left', 'press', '', 8, 0 }, { 'm', 'left', 'release', '', 8, 0 } }, { foldcolumn = '2' } },
    { 'nomouse-drag', { { 'm', 'left', 'press', '', 1, 2 }, { 'm', 'left', 'drag', '', 3, 8 } }, { mouse = '' } },
    { 'tabline-noclickdef', { { 'm', 'left', 'press', '', 0, 40 }, { 'm', 'left', 'release', '', 0, 40 } }, { showtabline = 2 } },
  }

  local aborted = 0
  for _, p in ipairs(PROBES) do
    local c = child_new()
    c:lua('RESET(' .. tolua(p[3] or {}) .. ')')
    run_events(c, p[2], 'wait')
    -- TWO liveness questions, and they are not the same one. The first
    -- asks whether the INPUT killed the editor; the second asks whether
    -- reading the answer did, because `getmousepos()` is itself in the
    -- file under test and `f_getmousepos` is where an out-of-range
    -- `mouse_col` lands.
    local alive = c:lua('return 1')
    local ans = c:lua('return ANS({})')
    local verdict
    if type(ans) == 'table' then
      verdict = { live = true, ans = ans }
    else
      aborted = aborted + 1
      -- `jobwait` first: the dying words arrive on stderr after the
      -- channel has already gone, so reading `c.err` before reaping is
      -- a race that answers an empty list about half the time.
      c:stop()
      verdict = {
        live = false,
        input_survived = (alive == 1),
        why = tostring(ans),
        said = said(c.err),
      }
    end
    label_once('s91/' .. p[1])
    emit('s91/' .. p[1], 'X', esc(scrub(ins(verdict))))
    struct('s91/' .. p[1], verdict)
    c:stop()
  end
  emit('s91', 'groups', string.format('cases=%d aborted=%d', #PROBES, aborted))
end)

structfd:close()
emit('##', 'TOTAL', string.format('rows=%d', rows))
