-- termsweep -- the twenty-first baselined differential.  Driven by
-- termsweep.sh, which builds the sandbox, pins $HOME/
-- $TMPDIR/$PATH and does the scrubs only the shell can see.  Read that
-- header first.
--
-- The subsystem is terminal.rs (820 lines) and terminal/{mode, refresh,
-- callbacks, input, scrollback, termrequest}.rs (2,334 more): the half
-- of B21 that vtsweep cannot see, because vtsweep drives the emulator
-- through `ffi.C` and never opens a terminal at all.
--
-- ADOPTED from the phase-15 tool `1785449630-termsweep.{sh,lua}`
-- (B8/B9, twenty scenarios, never baselined).  All twenty scenario
-- names survive -- short, overflow, trim, retrim, regrow, altscreen,
-- overwrite, wide, attrs, colors, hyperlink, title, cursor, wrap,
-- clear, region, narrow, tall, regrow_window, wipe -- and every one of
-- them is a case label below.  What changed:
--
--   * The DRIVE.  The old tool ran each scenario as `jobstart(pty)` in
--     its own process and settled it with `vim.wait(400)`, `redraw`,
--     `vim.wait(200)`.  That is a race with a shell, a fork and a
--     10 ms refresh timer, which is why it was never baselined.  The
--     scenarios whose subject is the emulator, the scrollback and the
--     refresh are driven by `nvim_open_term` + `nvim_chan_send`
--     instead -- the entry `terminal_receive` takes from a pty read,
--     with no child in the answer -- and the deferred refresh is
--     forced SYNCHRONOUSLY by shrinking `'scrollback'` (B21-2's
--     `termchurn` lever: `did_set_scrollback` calls
--     `on_scrollback_option_changed` only on a shrink).
--   * The PTY PATH IS STILL DRIVEN, in t7, because it is real code the
--     sweep exists to cover -- `terminal_open`'s job, the
--     `[Process exited]` extmark, TermOpen/TermClose, `:terminal`
--     itself.  It is made deterministic by SYNCING ON EFFECTS
--     (`jobwait` for the exit, `vim.wait` on the line that proves the
--     output arrived), never on a duration.
--   * The ANSWER.  The old tool wrote one file per scenario holding
--     the buffer lines.  This one writes one canonical report plus a
--     canonical JSON, with per-section row counts, and answers the
--     scrollback split, the cursor, the termprops, the reply bytes,
--     the TermRequest payloads and the per-cell ATTRIBUTES.
--   * The ATTRIBUTE SECTION is new (t3/t4) and is the gap the B21
--     survey named: `terminal_get_line_attributes` is read by NO
--     oracle in the tree.  See the section header for the hlstate rule.
--
-- Sections:
--   t0  canary      the world every later section assumes, and the
--                   refresh lever's own effect assertions
--   t1  sb          scrollback/refresh bookkeeping: 13 of the 20
--   t2  geom        window geometry -> terminal_check_size -> resize,
--                   pop and push: narrow, tall, regrow_window
--   t3  attr        terminal_get_line_attributes, per cell, headless
--   t4  attrui      the same, in an --embed child with a UI ATTACHED
--                   (`ui_rgb_attached()` is a different arm)
--   t5  input       terminal-mode keys and mouse -> the bytes the
--                   child would receive (terminal/input.rs, mode.rs)
--   t6  request     OSC/DCS termprops, `TermRequest`, the reply writer,
--                   theme updates (terminal/termrequest.rs)
--   t7  pty         the real pty path: `:terminal`, jobstart(term),
--                   TermOpen/TermClose, [Process exited], wipe
--   t91 abortprobe  the inputs that may kill the editor, one child each
--
-- Every section ends with a `## <name> rows=N` line.  A sweep that goes
-- silently empty otherwise looks exactly like a healthy one -- and this
-- family has two ways to go empty that cost real time to find: a
-- `pcall` around a `vim.cmd` that raised swallows the case and leaves
-- the row shape intact, and under `-l` (which this sweep does NOT use)
-- `full_screen` is false, `'columns'` does not resize the grid, and
-- every geometry answer collapses onto the compiled default.

local work = assert(os.getenv('TERM_WORK'), 'TERM_WORK unset')
local runtime = os.getenv('VIMRUNTIME') or ''
local script = debug.getinfo(1, 'S').source:sub(2)

local only = os.getenv('TERMSWEEP_ONLY')
if only == '' then
  only = nil
end
local trace = os.getenv('TERMSWEEP_TRACE') == '1'

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
  -- `term://<cwd>//<pid>:<cmd>` is a buffer name and a b:term_title.
  text = text:gsub('term://([^%s]-)//%d+:', 'term://%1//<PID>:')
  text = text:gsub('nvim%.%d+%.%d+', 'nvim.<PID>.<SEQ>')
  return text
end

local function cap(text, limit)
  limit = limit or 400
  if #text <= limit then
    return text
  end
  return text:sub(1, limit) .. string.format('...<+%d>', #text - limit)
end

--- Escape to one printable line.  Terminal output is full of control
--- bytes and the fixtures are deliberately multibyte.
local function esc(bytes)
  return (tostring(bytes):gsub('[%c\128-\255\\ ]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

--- A stable digest, so a 400-line scenario answers one row instead of
--- four hundred.  FNV-1a, 32-bit, in pure Lua: the full text is in the
--- struct artifact, and this is what makes the REPORT diffable.
local bit = require('bit')
local function digest(text)
  local h = bit.tobit(2166136261)
  for i = 1, #text do
    h = bit.bxor(h, text:byte(i))
    h = bit.tobit(h * 16777619)
  end
  return bit.tohex(h)
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
  assert(io.open(assert(os.getenv('TERM_STRUCT'), 'TERM_STRUCT unset'), 'w'))

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

local function errtext(res)
  local s = tostring(res)
  s = s:gsub('\nstack traceback:.*$', '')
  s = s:gsub('^[^\n]-termsweep%.lua:%d+: ', '')
  s = s:gsub('^%[string "[^"]*"%]:%d+: ', '')
  s = s:gsub('\r?\n', ' | ')
  return cap(scrub(s))
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

--- One case: a text row and a JSON row under the same label.  The text
--- row is `k=v` in sorted key order, values escaped to one line.
local function ans(label, answer)
  label_once(label)
  local keys = {}
  for k in pairs(answer) do
    keys[#keys + 1] = k
  end
  table.sort(keys)
  local parts = {}
  for _, k in ipairs(keys) do
    local v = answer[k]
    if type(v) == 'table' then
      v = table.concat(
        vim.tbl_map(function(x)
          return esc(scrub(tostring(x)))
        end, v),
        ','
      )
    else
      v = esc(scrub(tostring(v)))
    end
    parts[#parts + 1] = k .. '=' .. cap(v, 300)
  end
  emit(label, table.concat(parts, ' '))
  struct(label, answer)
end

-- =============================================================== drive
-- `nvim_open_term` + `nvim_chan_send` is the pty read path minus the
-- pty.  `poke()` is the only synchronous door into `refresh_terminal`
-- reachable from script, and every answer below is taken after one.
-- ====================================================================

local SB = 100

--- A fresh single-window terminal of a known geometry.
local function newterm(o)
  o = o or {}
  pcall(vim.cmd, 'silent! only!')
  pcall(vim.cmd, 'silent! tabonly!')
  vim.o.lines = o.lines or 24
  vim.o.columns = o.columns or 80
  vim.o.laststatus = 0
  vim.o.showtabline = 0
  vim.o.cmdheight = 1
  vim.o.equalalways = false
  vim.o.winminwidth = 0
  vim.o.winminheight = 0
  local buf = vim.api.nvim_create_buf(false, true)
  -- The vterm's width is the MAXIMUM over the windows showing the
  -- buffer (`terminal_check_size`), so the other half of a split must
  -- hold a DIFFERENT buffer or a narrow case silently measures the
  -- wide window instead.
  vim.api.nvim_win_set_buf(0, vim.api.nvim_create_buf(false, true))
  if o.cols then
    vim.cmd('vsplit')
    vim.cmd('wincmd l')
  end
  vim.api.nvim_win_set_buf(0, buf)
  if o.rows then
    vim.api.nvim_win_set_height(0, o.rows)
  end
  if o.cols then
    vim.api.nvim_win_set_width(0, o.cols)
  end
  local T = { buf = buf, replies = {}, reqs = {} }
  T.chan = vim.api.nvim_open_term(buf, {
    force_crlf = o.force_crlf or false,
    on_input = function(_, _, _, data)
      T.replies[#T.replies + 1] = data
    end,
  })
  vim.bo[buf].scrollback = o.scrollback or SB
  T.win = vim.api.nvim_get_current_win()
  return T
end

--- Force the deferred refresh.  Shrinking `'scrollback'` is the lever;
--- growing first keeps the value where the case put it.
local function poke(T)
  local sb = vim.bo[T.buf].scrollback
  local ok = pcall(function()
    vim.bo[T.buf].scrollback = sb + 1
  end)
  if not ok then
    -- Already at the option's maximum (E474 on the grow).  Shrink by
    -- one instead and put it back: the shrink is the event either way,
    -- and no buffer here is a million lines long, so nothing is
    -- trimmed by the detour.
    pcall(function()
      vim.bo[T.buf].scrollback = sb - 1
    end)
  end
  vim.bo[T.buf].scrollback = sb
end

local function send(T, bytes)
  vim.api.nvim_chan_send(T.chan, bytes)
  poke(T)
end

--- Wait for something the EVENT LOOP has to deliver -- a TermRequest
--- autocommand, a reply written back to `on_input`, a job's exit.  The
--- condition is the effect itself; the timeout is a failure report, not
--- a schedule.
local function until_(cond, ms)
  local ok = vim.wait(ms or 4000, cond, 1)
  return ok
end

--- The answer shape.  Everything here is a fact about the terminal
--- buffer that survives being read from a headless process.
local function state(T, extra)
  local lines = vim.api.nvim_buf_get_lines(T.buf, 0, -1, false)
  local joined = table.concat(lines, '\n')
  local a = {
    lines = #lines,
    bytes = #joined,
    sha = digest(joined),
    title = tostring(vim.b[T.buf].term_title),
    scrollback = vim.bo[T.buf].scrollback,
    modified = tostring(vim.bo[T.buf].modified),
    buftype = vim.bo[T.buf].buftype,
    chan = tostring(vim.bo[T.buf].channel ~= 0),
    cursor = table.concat(vim.api.nvim_win_get_cursor(T.win), ','),
    w0 = vim.fn.line('w0', T.win),
  }
  -- The whole text goes to the struct; the report keeps the head and
  -- the tail, which is where a push/pop/trim bug shows.
  a.text = lines
  local head, tail = {}, {}
  for i = 1, math.min(4, #lines) do
    head[i] = lines[i]
  end
  for i = math.max(1, #lines - 3), #lines do
    tail[#tail + 1] = lines[i]
  end
  a.head = head
  a.tail = tail
  for k, v in pairs(extra or {}) do
    a[k] = v
  end
  return a
end

--- A payload builder: `\r\n` because a pty terminal gets CRLF and
--- `force_crlf` is off, exactly as `terminal_receive` sees it.
local function nlines(prefix, n)
  local parts = {}
  for i = 1, n do
    parts[#parts + 1] = string.format('%s %d\r\n', prefix, i)
  end
  return table.concat(parts)
end

--- WHERE THE SCREEN STARTS.  A scrollback pop or push moves lines
--- between the emulator's ring and its screen without changing the
--- buffer's line COUNT -- both halves are buffer lines -- so a line
--- count is blind to `term_sb_pop` and `refresh_size`.  This writes a
--- marker into the top SCREEN row (between DECSC and DECRC, so the
--- cursor the output was using is put back) and answers the buffer
--- line it landed on, which IS the number of scrollback lines.
local function topmark(T, tag)
  vim.api.nvim_chan_send(T.chan, '\0277\27[H\27[2K' .. tag .. '\0278')
  poke(T)
  local lines = vim.api.nvim_buf_get_lines(T.buf, 0, -1, false)
  for i, l in ipairs(lines) do
    if l:find(tag, 1, true) then
      return i, #lines
    end
  end
  return -1, #lines
end

-- =============================================================== t0

section('t0-canary', function()
  local T = newterm()
  ans('t0/fresh', state(T))

  -- The refresh lever's own contract.  If a rewrite made the refresh
  -- eager, or made the poke inert, these three numbers move and every
  -- later section's determinism argument is void.  B21-2 measured the
  -- same three: 12 / 12 / 24 on a 12-row terminal.
  local L = newterm({ rows = 12, scrollback = SB })
  vim.api.nvim_chan_send(L.chan, nlines('poke', 24))
  local n0 = vim.api.nvim_buf_line_count(L.buf)
  vim.bo[L.buf].scrollback = SB + 1
  local n1 = vim.api.nvim_buf_line_count(L.buf)
  vim.bo[L.buf].scrollback = SB
  local n2 = vim.api.nvim_buf_line_count(L.buf)
  ans('t0/lever', { after_send = n0, after_grow = n1, after_shrink = n2 })

  -- The terminal's size comes from the WINDOW, through
  -- `terminal_check_size`, which refuses only a zero.  Measured by
  -- where the emulator wraps, which is the only observable a headless
  -- process has for the vterm's width.
  for _, geom in ipairs({
    { 80, 23 },
    { 40, 10 },
    { 20, 6 },
    { 10, 3 },
    { 2, 2 },
    { 1, 1 },
  }) do
    local W = newterm({ cols = geom[1], rows = geom[2] })
    send(W, string.rep('W', 200) .. '\r\n')
    local lines = vim.api.nvim_buf_get_lines(W.buf, 0, -1, false)
    local widths = {}
    for _, l in ipairs(lines) do
      if #l > 0 then
        widths[#widths + 1] = #l
      end
    end
    ans(string.format('t0/size/c%dr%d', geom[1], geom[2]), {
      winwidth = vim.api.nvim_win_get_width(W.win),
      winheight = vim.api.nvim_win_get_height(W.win),
      lines = #lines,
      widths = widths,
    })
  end

  -- `'scrollback'` is clamped by the option machinery, not by the
  -- terminal; the terminal then trims to whatever survived.
  for _, v in ipairs({ -1, 0, 1, 2, 100001 }) do
    local S = newterm({ rows = 6 })
    vim.api.nvim_chan_send(S.chan, nlines('sb', 40))
    local ok, err = pcall(function()
      vim.bo[S.buf].scrollback = v
    end)
    poke(S)
    ans('t0/scrollback/' .. tostring(v), {
      set = tostring(ok),
      err = ok and '' or errtext(err),
      value = vim.bo[S.buf].scrollback,
      lines = vim.api.nvim_buf_line_count(S.buf),
    })
  end
end)

-- =============================================================== t1
-- The phase-15 scenarios whose subject is the buffer, the scrollback
-- and the refresh.  Thirteen of the twenty names live here.
-- ====================================================================

section('t1-sb', function()
  local cases = {
    -- Output shorter than the window: nothing reaches scrollback.
    { 'short', { rows = 10 }, 'one\r\ntwo\r\nthree\r\n' },
    -- More lines than the window holds: `term_sb_push`.
    { 'overflow', { rows = 10 }, nlines('line', 120) },
    -- Output far past `'scrollback'`: `adjust_scrollback` trims.
    { 'trim', { rows = 10, scrollback = 40 }, nlines('row', 400) },
    -- A window narrower than the output: the wrap arithmetic.
    { 'wrap', { rows = 10, cols = 40 }, string.rep('W', 200) .. '\r\n' .. string.rep('X', 41) .. '\r\n' },
    -- CR and BS overwrite in place rather than append.
    { 'overwrite', { rows = 10 }, 'aaaa\rbb\r\nxxxx\b\b\byy\r\n' },
    -- Wide, combining and emoji decide cell widths and schar packing.
    {
      'wide',
      { rows = 10 },
      'AB\228\184\173\230\150\135CD\r\ne\204\129 combining\r\n\240\159\142\137 emoji\r\n',
    },
    -- OSC 0 sets the title, which lands in `b:term_title`.
    { 'title', { rows = 10 }, '\27]0;probe-title\7body\r\n' },
    -- Cursor shape and visibility termprops.
    { 'cursor', { rows = 10 }, '\27[3 qx\27[?25l\27[?25h\r\nafter\r\n' },
    -- `\27[3J` clears the scrollback (`term_sb_clear`), `\27[2J` the
    -- screen.
    { 'clear', { rows = 10 }, nlines('c', 40) .. '\27[3J\27[2J\27[Hafter\r\n' },
    -- A scroll region moves rows WITHOUT pushing scrollback.
    { 'region', { rows = 10 }, nlines('r', 20) .. '\27[5;10r\27[10;1H' .. nlines('s', 8) .. '\27[r' },
    -- The alternate screen hides the scrollback and leaving pops it
    -- back: `term_sb_pop`.
    {
      'altscreen',
      { rows = 10 },
      nlines('a', 50) .. '\27[?1049hALT1\r\nALT2\r\n\27[?1049ldone\r\n',
    },
  }
  for _, c in ipairs(cases) do
    local T = newterm(c[2])
    send(T, c[3])
    ans('t1/' .. c[1], state(T))
  end

  -- `'scrollback'` lowered after the fact re-trims an existing buffer.
  local R = newterm({ rows = 10 })
  send(R, nlines('n', 200))
  local before = vim.api.nvim_buf_line_count(R.buf)
  vim.bo[R.buf].scrollback = 25
  poke(R)
  ans('t1/retrim', state(R, { before = before }))

  -- Raised, it must not invent lines.
  local G = newterm({ rows = 10, scrollback = 30 })
  send(G, nlines('g', 60))
  local kept = vim.api.nvim_buf_line_count(G.buf)
  vim.bo[G.buf].scrollback = 500
  poke(G)
  ans('t1/regrow', state(G, { before = kept }))

  -- `CSI 3 J` (clear scrollback) WHILE ON THE ALTERNATE SCREEN.  The
  -- ring belongs to the screen underneath and `term_sb_clear` refuses
  -- there; nothing else in the sweep tells the two refusals apart --
  -- `t1/clear` clears a NON-empty ring on the primary screen, and the
  -- `in_altscreen` conjunct in front of it was invisible until this
  -- case existed (termmutate `cb-altscreen`).
  local AC = newterm({ rows = 8 })
  send(AC, nlines('keep', 30))
  local kept0 = vim.api.nvim_buf_line_count(AC.buf)
  send(AC, '\27[?1049h')
  send(AC, '\27[3J\27[2J\27[Halt body\r\n')
  local inalt0 = vim.api.nvim_buf_line_count(AC.buf)
  send(AC, '\27[?1049l')
  ans('t1/altscreen-clear', state(AC, { before = kept0, inalt = inalt0 }))

  -- A second alternate-screen round trip, with output in between: the
  -- pop must not resurrect what the first push took.
  local A = newterm({ rows = 8 })
  send(A, nlines('p', 30))
  send(A, '\27[?1049h')
  local inalt = vim.api.nvim_buf_line_count(A.buf)
  send(A, nlines('alt', 12))
  send(A, '\27[?1049l')
  ans('t1/altscreen2', state(A, { inalt = inalt }))
end)

-- =============================================================== t2
-- Window geometry.  `terminal_check_size` sizes the vterm from the
-- WIDEST window showing the buffer; a resize runs `vterm_set_size`,
-- `refresh_size` and, growing, `term_sb_pop`.
-- ====================================================================

section('t2-geom', function()
  -- narrow: a narrow window makes the wrapping arithmetic work.
  local N = newterm({ cols = 20, rows = 6 })
  send(N, nlines('narrow line', 40))
  local nat, ntot = topmark(N, 'NTOP')
  ans('t2/narrow', state(N, { screen_top = nat, total = ntot }))

  -- tall: a tall run into a tiny window is maximum push pressure.
  local T = newterm({ rows = 4 })
  send(T, nlines('t', 300))
  local tat, ttot = topmark(T, 'TTOP')
  ans('t2/tall', state(T, { screen_top = tat, total = ttot }))

  -- regrow_window: growing the window POPS scrollback back onto the
  -- screen and shrinking PUSHES it.  The buffer's line count is
  -- invariant across both -- the two halves are the same lines -- so
  -- the answer is where the screen starts, per height.
  local G = newterm({ rows = 6 })
  send(G, nlines('w', 80))
  local splits = {}
  for _, h in ipairs({ 6, 18, 10, 4, 20 }) do
    vim.api.nvim_win_set_height(G.win, h)
    vim.cmd('redraw')
    poke(G)
    local at, tot = topmark(G, 'GTOP' .. h)
    splits[#splits + 1] = string.format('h%d:top%d/%d', h, at, tot)
  end
  ans('t2/regrow_window', state(G, { splits = splits }))

  -- Reflow: the emulator re-wraps the rows that are still ON SCREEN
  -- when the width changes (scrollback rows keep the width they were
  -- pushed at), so the fixture is three long lines in a tall window.
  for _, w in ipairs({ 60, 30, 15, 45 }) do
    local W = newterm({ cols = 40, rows = 16 })
    send(W, string.rep('R', 100) .. '\r\n' .. string.rep('S', 55) .. '\r\nrf tail\r\n')
    vim.api.nvim_win_set_width(W.win, w)
    vim.cmd('redraw')
    poke(W)
    ans('t2/reflow/' .. w, state(W, { width = vim.api.nvim_win_get_width(W.win) }))
  end

  -- Two windows on ONE terminal: the size is the MAXIMUM, so the
  -- narrow one does not shrink the emulator.
  local M = newterm({ rows = 10 })
  send(M, nlines('m', 20))
  vim.cmd('vsplit')
  vim.api.nvim_win_set_width(0, 12)
  vim.cmd('redraw')
  poke(M)
  local wins = {}
  for _, w in ipairs(vim.api.nvim_list_wins()) do
    wins[#wins + 1] = vim.api.nvim_win_get_width(w)
  end
  send(M, string.rep('M', 70) .. '\r\n')
  ans('t2/twowin', state(M, { widths = wins }))

  -- The terminal shown in NO window keeps the size it had.
  local H = newterm({ rows = 10 })
  send(H, nlines('h', 12))
  vim.cmd('enew')
  vim.cmd('redraw')
  vim.api.nvim_chan_send(H.chan, string.rep('H', 90) .. '\r\n')
  poke(H)
  ans('t2/hidden', state(H))
end)

-- ============================================================== child
-- Every section that needs the main input loop, a UI, or a process it
-- can afford to lose drives one `--embed` child over
-- `jobstart(rpc = true)`.  Shared by t3 (the palette case, see there),
-- t4, t5 and t91.
-- ====================================================================

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
    'set noswapfile shell=/bin/sh',
  }
  for _, a in ipairs(args or {}) do
    argv[#argv + 1] = a
  end
  local errlines = {}
  local chan = vim.fn.jobstart(argv, {
    rpc = true,
    cwd = work,
    clear_env = true,
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

function Child:lua(code)
  if self.dead then
    return 'DEAD'
  end
  local ok, res = pcall(vim.rpcrequest, self.chan, 'nvim_exec_lua', code, {})
  if ok then
    return res
  end
  local text = errtext(res)
  if
    text:match('closed by the peer')
    or text:match('[Ii]nvalid channel')
    or text:match('channel closed')
  then
    self.dead = true
  end
  return 'RPCERR ' .. text
end

function Child:key(keys)
  if self.dead then
    return 'DEAD'
  end
  local ok, err = pcall(vim.rpcrequest, self.chan, 'nvim_input', keys)
  return ok and 'ok' or ('ERR ' .. errtext(err))
end

--- A KEYLESS barrier: one deferred request, answered.  `nvim_input` is
--- a FAST call and `nvim_exec_lua` is DEFERRED to the same loop that
--- takes the typeahead first, so an answer to this proves every key
--- sent before it has been executed.  It must stay keyless -- a
--- `<Cmd>` marker would itself be a key in the stream this section is
--- measuring.
function Child:barrier()
  return self:lua('return 1') == 1
end

function Child:said()
  local out = {}
  for _, line in ipairs(self.err or {}) do
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

function Child:stop()
  pcall(vim.fn.jobstop, self.chan)
  pcall(vim.fn.jobwait, { self.chan }, 10000)
end

--- Installed in every child: the same drive as the parent's.
local PRELUDE = [[
_G.TS = {}
function _G.TERM_NEW(o)
  o = o or {}
  pcall(vim.cmd, 'silent! only!')
  vim.o.lines = o.lines or 24
  vim.o.columns = o.columns or 80
  vim.o.laststatus = 0
  vim.o.showtabline = 0
  vim.o.equalalways = false
  vim.o.winminwidth = 0
  vim.o.shell = '/bin/sh'
  local buf = vim.api.nvim_create_buf(false, true)
  -- The other half of a split must hold a DIFFERENT buffer: the vterm's
  -- width is the MAXIMUM over the windows showing it.
  vim.api.nvim_win_set_buf(0, vim.api.nvim_create_buf(false, true))
  if o.cols then vim.cmd('vsplit') vim.cmd('wincmd l') end
  vim.api.nvim_win_set_buf(0, buf)
  if o.rows then vim.api.nvim_win_set_height(0, o.rows) end
  if o.cols then vim.api.nvim_win_set_width(0, o.cols) end
  _G.TS.buf = buf
  _G.TS.input = {}
  _G.TS.chan = vim.api.nvim_open_term(buf, { force_crlf = false,
    on_input = function(_, _, _, data) _G.TS.input[#_G.TS.input + 1] = data end })
  vim.bo[buf].scrollback = 100
  return buf
end
function _G.TERM_POKE()
  local sb = vim.bo[_G.TS.buf].scrollback
  vim.bo[_G.TS.buf].scrollback = sb + 1
  vim.bo[_G.TS.buf].scrollback = sb
end
function _G.TERM_SEND(bytes)
  vim.api.nvim_chan_send(_G.TS.chan, bytes)
  _G.TERM_POKE()
end
function _G.TERM_CELLS(row, cols)
  local parts = {}
  for col = 0, cols - 1 do
    local c = vim.api.nvim__inspect_cell(1, row, col)
    local ch, d = c[1] or '', c[2]
    local keys = {}
    if type(d) == 'table' then
      for k in pairs(d) do keys[#keys + 1] = k end
      table.sort(keys)
    end
    local bits = {}
    for _, k in ipairs(keys) do bits[#bits + 1] = k .. '=' .. tostring(d[k]) end
    parts[#parts + 1] = (ch == '' and '_' or ch) .. '{' .. table.concat(bits, ',') .. '}'
  end
  return parts
end
]]

local function child_new(args)
  local c = child_start(args)
  c:lua(PRELUDE)
  return c
end

-- =============================================================== t3
-- `terminal_get_line_attributes`, which no oracle in the tree read
-- before this section.  Every visible line of a terminal buffer goes
-- through it on every redraw; the answer is the per-column highlight
-- the screen ends up holding.
--
-- ONE RULE, and it is not optional.  The FIRST `nvim__inspect_cell`
-- call in a process enables hlstate (`highlight_use_hlstate()` returns
-- true once and CLEARS THE HIGHLIGHT TABLES on the way), so the grid
-- is left holding attribute ids that have just been invalidated and
-- every read after the first answers a different definition for the
-- same id.  Warm it with one throwaway call and a `redraw!` before
-- reading anything, or the section answers plausible garbage.
-- ====================================================================

local ATTR_CASES = {
  -- attrs: the SGR alphabet the drawing path has a bit for.
  { 'attrs/bold', '\27[1mBOLD\27[0m' },
  { 'attrs/italic', '\27[3mITAL\27[0m' },
  { 'attrs/underline', '\27[4mUNDR\27[0m' },
  { 'attrs/undercurl', '\27[4:3mCURL\27[0m' },
  { 'attrs/underdouble', '\27[4:2mDBLU\27[0m' },
  { 'attrs/blink', '\27[5mBLNK\27[0m' },
  { 'attrs/reverse', '\27[7mREVR\27[0m' },
  { 'attrs/conceal', '\27[8mCONC\27[0m' },
  { 'attrs/strike', '\27[9mSTRK\27[0m' },
  { 'attrs/overline', '\27[53mOVER\27[0m' },
  { 'attrs/dim', '\27[2mDIMM\27[0m' },
  { 'attrs/mixed', '\27[1;3;4;7mMIXD\27[0m' },
  -- colors: indexed, bright, 256 and 24-bit take different arms of the
  -- fg/bg resolution, and `color_set` decides indexed-vs-rgb.
  { 'colors/ansi', '\27[31mRED\27[0m\27[42mGRN\27[0m' },
  { 'colors/bright', '\27[91mBRT\27[0m\27[102mBGB\27[0m' },
  { 'colors/idx256', '\27[38;5;208mIDX\27[0m\27[48;5;19mBGI\27[0m' },
  { 'colors/rgb', '\27[38;2;255;136;0mTRU\27[0m\27[48;2;0;16;64mBGT\27[0m' },
  { 'colors/default', '\27[39;49mDEF\27[0m' },
  { 'colors/reverse-rgb', '\27[7;38;2;10;20;30mRVR\27[0m' },
  -- hyperlink: `cell.uri` is combined over the colours.
  { 'hyperlink/plain', '\27]8;;https://example.com\27\\LINK\27]8;;\27\\ tail' },
  { 'hyperlink/coloured', '\27[34m\27]8;;https://e.org\27\\CLNK\27]8;;\27\\\27[0m' },
  -- wide and combining: a continuation cell has no attributes of its
  -- own and must not be given the next cell's.
  { 'wide/cjk', '\27[35m\228\184\173\230\150\135\27[0m x' },
  { 'wide/combining', '\27[36me\204\129xy\27[0m' },
}

--- Dump one line's worth of resolved attributes.
local function attrline(row, cols)
  local parts = {}
  for col = 0, cols - 1 do
    local c = vim.api.nvim__inspect_cell(1, row, col)
    local ch = c[1] or ''
    local d = c[2]
    local keys = {}
    if type(d) == 'table' then
      for k in pairs(d) do
        keys[#keys + 1] = k
      end
      table.sort(keys)
    end
    local bits = {}
    for _, k in ipairs(keys) do
      bits[#bits + 1] = k .. '=' .. tostring(d[k])
    end
    parts[#parts + 1] = (ch == '' and '_' or ch) .. '{' .. table.concat(bits, ',') .. '}'
  end
  return parts
end

section('t3-attr', function()
  local T = newterm({ rows = 12 })
  -- Warm hlstate before anything is read; see the section header.
  vim.api.nvim__inspect_cell(1, 0, 0)
  vim.cmd('redraw!')
  for _, c in ipairs(ATTR_CASES) do
    local W = newterm({ rows = 12 })
    send(W, c[2] .. '\r\n')
    vim.cmd('redraw!')
    ans('t3/' .. c[1], {
      cells = attrline(0, 10),
      line = vim.api.nvim_buf_get_lines(W.buf, 0, 1, false)[1] or '',
    })
  end

  -- A line that is SCROLLBACK rather than screen resolves through
  -- `fetch_cell`'s scrollback arm, whose cells may be short.
  local S = newterm({ rows = 6 })
  send(S, '\27[33m' .. nlines('sbattr', 20) .. '\27[0m')
  vim.cmd('redraw!')
  ans('t3/scrollback', {
    top = attrline(0, 8),
    mid = attrline(3, 8),
    lines = vim.api.nvim_buf_line_count(S.buf),
  })

  -- A user palette entry turns an INDEXED colour into an rgb one
  -- (`color_set`), which is the branch `terminal_get_line_attributes`
  -- reads per cell.
  --
  -- IN A CHILD, and that is not fastidiousness.  `terminal_open` reads
  -- `g:terminal_color_N` through `get_config_string`, whose `Object`
  -- comes from `dict_get_value` with `reuse_strdata = true` -- the
  -- string data POINTS AT THE VARIABLE'S OWN BYTES -- and this port
  -- (not upstream, which does not free it) calls `xfree` on it.  The
  -- variable is left dangling, so unsetting it, re-setting it, or
  -- tearing the process down corrupts the heap.  Setting one in THIS
  -- process would poison every case after it; `t91/palette-*` is the
  -- probe that records the defect.
  local PC = child_new()
  PC:lua([==[
    vim.g.terminal_color_1 = '#00ff88'
    _G.TERM_NEW({rows = 6})
    _G.TERM_SEND('\27[31mPAL\27[0m\27[38;5;9mBRT\27[0m\r\n')
    vim.api.nvim__inspect_cell(1, 0, 0)
    vim.cmd('redraw!')
  ]==])
  local pcells = PC:lua('return _G.TERM_CELLS(0, 8)')
  ans('t3/palette', { cells = type(pcells) == 'table' and pcells or { tostring(pcells) } })
  PC:stop()

  -- `:hi Terminal` is layered under the cell attributes.
  vim.cmd('hi Terminal guifg=#123456 guibg=#654321 ctermfg=5 ctermbg=6')
  local H = newterm({ rows = 6 })
  send(H, 'plain\27[32mGRN\27[0m\r\n')
  vim.cmd('redraw!')
  ans('t3/hi-terminal', { cells = attrline(0, 10) })
  vim.cmd('hi clear Terminal')
  _ = T
end)

-- =============================================================== t4
-- The same resolution with a UI ATTACHED.  `ui_rgb_attached()` is read
-- by the drawing path that consumes `term_attrs`
-- (`drawline/attrs.rs`), and no headless sweep has ever turned it on.
--
-- The child is `--embed --listen` over `jobstart(rpc = true)` for
-- control, plus a RAW socket channel for the UI: an RPC channel cannot
-- be a UI, because the child's first `redraw` NOTIFICATION resolves to
-- no API handler in this parent and the channel is closed (B19-1/B19-3).
-- Fourteen hand-encoded msgpack bytes are the whole client.
-- ====================================================================


section('t4-attrui', function()
  local sock = work .. '/ui.sock'
  os.remove(sock)
  local C = child_new({ '--listen', sock })

  local function u8(...)
    return string.char(...)
  end
  local function fixstr(s)
    return u8(0xa0 + #s) .. s
  end
  -- [0, 1, "nvim_ui_attach", [80, 24, {"rgb": true}]]
  local attach = u8(0x94)
    .. u8(0x00)
    .. u8(0x01)
    .. fixstr('nvim_ui_attach')
    .. u8(0x93)
    .. u8(0x50)
    .. u8(0x18)
    .. u8(0x81)
    .. fixstr('rgb')
    .. u8(0xc3)

  local uichan = -1
  local ok = pcall(function()
    uichan = vim.fn.sockconnect('pipe', sock, { rpc = false, on_data = function() end })
  end)
  if not ok or uichan <= 0 then
    ans('t4/ui', { attached = 'SOCKCONNECT FAILED' })
    C:stop()
    return
  end
  vim.fn.chansend(uichan, attach)
  -- The attach is asynchronous; the first query that answers proves it
  -- landed.  A fixed wait would be a race on a loaded machine.
  local uis
  for _ = 1, 200 do
    uis = C:lua('return {n = #vim.api.nvim_list_uis(), rgb = (vim.api.nvim_list_uis()[1] or {}).rgb}')
    if type(uis) == 'table' and uis.n and uis.n > 0 then
      break
    end
    vim.wait(10)
  end
  ans('t4/ui/attached', { n = type(uis) == 'table' and uis.n or 'none', rgb = type(uis) == 'table' and tostring(uis.rgb) or 'none' })

  C:lua('vim.api.nvim__inspect_cell(1, 0, 0) vim.cmd("redraw!")')
  for _, c in ipairs(ATTR_CASES) do
    C:lua('_G.TERM_NEW({rows = 12})')
    C:lua(string.format('_G.TERM_SEND(%q .. "\\r\\n") vim.cmd("redraw!")', c[2]))
    local cells = C:lua('return _G.TERM_CELLS(0, 10)')
    ans('t4/' .. c[1], { cells = type(cells) == 'table' and cells or { tostring(cells) } })
  end

  -- 'termguicolors' with a UI attached is the other half of the same
  -- decision.
  C:lua('vim.o.termguicolors = true')
  for _, c in ipairs({ ATTR_CASES[13], ATTR_CASES[15], ATTR_CASES[16] }) do
    C:lua('_G.TERM_NEW({rows = 12})')
    C:lua(string.format('_G.TERM_SEND(%q .. "\\r\\n") vim.cmd("redraw!")', c[2]))
    local cells = C:lua('return _G.TERM_CELLS(0, 10)')
    ans('t4/tgc/' .. c[1]:gsub('/', '-'), { cells = type(cells) == 'table' and cells or { tostring(cells) } })
  end

  pcall(vim.fn.chanclose, uichan)
  C:stop()
end)

-- =============================================================== t5
-- Terminal-mode input: `terminal_send_key`, the mouse encoder and the
-- `'termmode'`/mode machinery in terminal/{input,mode}.rs.  The answer
-- is the exact byte string the child process would have received,
-- captured by `nvim_open_term`'s `on_input` -- so this section reads
-- the encoder directly rather than through a shell.
--
-- Keys need the main input loop, so the whole section is one `--embed`
-- child.  `nvim_input` is FAST and `nvim_exec_lua` DEFERRED; the
-- barrier is keyless.
-- ====================================================================

section('t5-input', function()
  local C = child_new()
  C:lua([[
    _G.TERM_NEW({rows = 10})
    vim.api.nvim_set_current_buf(_G.TS.buf)
    -- `'mouse'` gates the dispatch entirely; the default value does not
    -- include terminal mode, so without this every mouse case below
    -- answers an empty byte string and looks healthy.
    vim.o.mouse = 'a'
    vim.o.mousemodel = 'extend'
  ]])

  local function drive(label, keys, pre)
    C:lua('_G.TS.input = {} ' .. (pre or ''))
    for _, k in ipairs(keys) do
      C:key(k)
    end
    C:barrier()
    local got = C:lua('return {input = _G.TS.input, mode = vim.fn.mode(), tmode = vim.b[_G.TS.buf].term_mode}')
    if type(got) ~= 'table' then
      ans(label, { got = tostring(got) })
      return
    end
    ans(label, {
      input = got.input or {},
      mode = tostring(got.mode),
      term_mode = tostring(got.tmode),
    })
  end

  -- Entering terminal mode, then the plain alphabet.
  drive('t5/enter', { 'i' })
  drive('t5/ascii', { 'abc' })
  drive('t5/ctrl', { '<C-a><C-c><C-d><C-z>' })
  drive('t5/esc', { '<Esc>' })
  for _, k in ipairs({
    '<CR>',
    '<Tab>',
    '<BS>',
    '<Del>',
    '<Up>',
    '<Down>',
    '<Left>',
    '<Right>',
    '<Home>',
    '<End>',
    '<PageUp>',
    '<PageDown>',
    '<F1>',
    '<F12>',
    '<S-Tab>',
    '<M-x>',
    '<C-Left>',
    '<S-Up>',
  }) do
    drive('t5/key/' .. k:gsub('[<>]', ''), { k })
  end
  -- Multibyte and a paste.
  drive('t5/utf8', { '\228\184\173' })
  drive(
    't5/paste',
    {},
    [[
      vim.api.nvim_paste('pasted\nline2', false, -1)
      vim.api.nvim_paste('pasted\nline2', false, 1)
      vim.api.nvim_paste('', false, 2)
      vim.api.nvim_paste('', false, 3)
    ]]
  )

  -- Leaving terminal mode with the default `<C-\><C-n>`, then a normal
  -- key, which must NOT reach the child.
  drive('t5/leave', { '<C-\\><C-n>' })
  drive('t5/normal-after-leave', { 'j' })

  -- terminal/mode.rs: which keys enter terminal mode, what the mode
  -- events are, and the one-shot Normal escape.
  local function modecase(label, pre, keys)
    C:lua([[
      _G.TERM_NEW({rows = 10})
      vim.api.nvim_set_current_buf(_G.TS.buf)
      -- The last line is DELIBERATELY unterminated and the fixture is
      -- taller than the window: the cursor then sits mid-line and
      -- mid-buffer, which is what makes `terminal_check_cursor`'s
      -- normal-mode column step and its topline arithmetic visible.
      for i = 1, 14 do _G.TERM_SEND('mode fixture ' .. i .. '\r\n') end
      -- ... and the cursor is then walked BACK into the middle of it.
      -- At the end of the line `coladvance` clamps to the last
      -- character anyway, so the Normal-mode step of one column is
      -- invisible there (termmutate `md-normal-col`).
      _G.TERM_SEND('promptxy$ tail text\27[8D')
      _G.EV = {}
      for _, e in ipairs({'TermEnter', 'TermLeave'}) do
        vim.api.nvim_create_autocmd(e, { buffer = _G.TS.buf, callback = function()
          _G.EV[#_G.EV + 1] = e .. ':' .. vim.fn.mode()
        end })
      end
    ]] .. (pre or ''))
    C:barrier()
    for _, k in ipairs(keys) do
      C:key(k)
      C:barrier()
    end
    local got = C:lua([[
      return { mode = vim.fn.mode(1), ev = _G.EV, input = _G.TS.input,
               cur = vim.api.nvim_win_get_cursor(0), w0 = vim.fn.line('w0'),
               lines = vim.api.nvim_buf_line_count(_G.TS.buf) }
    ]])
    if type(got) ~= 'table' then
      ans(label, { got = tostring(got) })
      return
    end
    ans(label, {
      mode = tostring(got.mode),
      events = got.ev or {},
      input = got.input or {},
      cursor = type(got.cur) == 'table' and table.concat(got.cur, ',') or tostring(got.cur),
      w0 = got.w0,
      lines = got.lines,
    })
    C:key('<C-\\><C-n>')
    C:barrier()
  end

  modecase('t5/mode/i', nil, { 'i' })
  modecase('t5/mode/a', nil, { 'a' })
  modecase('t5/mode/A', nil, { 'A' })
  modecase('t5/mode/I', nil, { 'I' })
  modecase('t5/mode/startinsert', 'vim.cmd("startinsert")', {})
  modecase('t5/mode/oneshot', nil, { 'i', '<C-\\><C-o>', 'ggx' })
  modecase('t5/mode/scroll-normal', nil, { 'i', '<C-\\><C-n>', 'gg', 'G' })
  modecase('t5/mode/cmdline', nil, { 'i', '<C-\\><C-n>', ':let g:x=1<CR>' })

  -- Mouse events in terminal mode are forwarded only when the emulator
  -- asked for them.
  C:lua('vim.api.nvim_set_current_buf(_G.TS.buf)')
  C:key('i')
  C:barrier()
  --- Terminal mode has to be re-entered for EVERY mouse case: a click
  --- the emulator did not ask for is how the user leaves terminal mode,
  --- so the case before this one has already left it -- and a click in
  --- Normal mode starts Visual, which is what the first draft measured
  --- four times over.
  local function enter_term()
    C:key('<C-\\><C-n>')
    C:barrier()
    C:lua('vim.api.nvim_set_current_buf(_G.TS.buf)')
    C:key('i')
    C:barrier()
    return C:lua('return vim.fn.mode()')
  end

  local function mouse(label, seq, mode)
    -- A FRESH terminal per case.  The DECSET modes are sticky, so
    -- reusing one would make every case depend on the ones before it
    -- and an inserted case would renumber the answers below it.
    C:lua('_G.TERM_NEW({rows = 10}) vim.api.nvim_set_current_buf(_G.TS.buf) vim.o.mouse = "a"')
    local entered = enter_term()
    C:lua('_G.TS.input = {} ' .. (mode and ('_G.TERM_SEND("' .. mode .. '")') or ''))
    for _, ev in ipairs(seq) do
      pcall(
        vim.rpcrequest,
        C.chan,
        'nvim_input_mouse',
        ev[1],
        ev[2],
        ev[3] or '',
        0,
        ev[4],
        ev[5]
      )
      -- A barrier PER EVENT, not per case.  `nvim_input_mouse` is a
      -- FAST call, and `do_mouse` coalesces a queued move or drag with
      -- the one it is handling (`vpeekc()` + `safe_vgetc()`), so two
      -- events pushed back to back are sometimes ONE report and
      -- sometimes two -- which is exactly what two runs of this
      -- section disagreed about before this line existed.  It must
      -- stay KEYLESS: a `<Cmd>` marker would be decoded by that same
      -- `safe_vgetc` and would reset `mod_mask` for the event in
      -- flight (B19-3).
      C:barrier()
    end
    local got = C:lua('return {input = _G.TS.input, mode = vim.fn.mode(), buf = vim.api.nvim_get_current_buf() == _G.TS.buf, mouse = vim.o.mouse}')
    if type(got) ~= 'table' then
      ans(label, { input = { tostring(got) } })
      return
    end
    ans(label, {
      input = got.input or {},
      entered = tostring(entered),
      mode = tostring(got.mode),
      onbuf = tostring(got.buf),
    })
  end
  mouse('t5/mouse/off', { { 'left', 'press', '', 3, 5 }, { 'left', 'release', '', 3, 5 } })
  -- DECSET 9 (X10 mouse reporting) is NOT one of the modes this
  -- emulator turns into `VTERM_PROP_MOUSE`, so it forwards nothing and
  -- the click leaves terminal mode exactly as `off` does.  Kept as a
  -- case because that is the answer, not because it reports.
  mouse('t5/mouse/decset9', {
    { 'left', 'press', '', 3, 5 },
    { 'left', 'release', '', 3, 5 },
  }, '\\27[?9h')
  -- 1000 alone: reporting on, protocol left at X10 -- the `\27[M`
  -- encoder, which nothing else here reaches.
  mouse('t5/mouse/x10', {
    { 'left', 'press', '', 3, 5 },
    { 'middle', 'press', 'C', 4, 6 },
    { 'left', 'release', '', 3, 5 },
  }, '\\27[?1000h')
  -- 1003: every motion is reported.
  mouse('t5/mouse/move', {
    { 'move', '', '', 2, 2 },
    { 'move', '', '', 2, 3 },
  }, '\\27[?1003h\\27[?1006h')
  mouse('t5/mouse/sgr', {
    { 'left', 'press', '', 2, 4 },
    { 'left', 'drag', '', 2, 6 },
    { 'left', 'release', '', 2, 6 },
    { 'right', 'press', 'S', 1, 1 },
  }, '\\27[?1002h\\27[?1006h')
  mouse('t5/mouse/wheel', {
    { 'wheel', 'up', '', 4, 4 },
    { 'wheel', 'down', '', 4, 4 },
  }, '\\27[?1000h\\27[?1006h')

  ans('t5/said', { stderr = C:said() })
  C:stop()
end)

-- =============================================================== t6
-- OSC/DCS termprops, the `TermRequest` autocommand and the reply
-- writer -- terminal/termrequest.rs and the `on_osc`/`on_dcs` half of
-- terminal/callbacks.rs.  Both are delivered through the EVENT LOOP,
-- so every case waits for its own effect.
-- ====================================================================

--- Installed in t6's child: the clipboard provider, the `TermRequest`
--- recorder and one case driver.
local T6_PRELUDE = [==[
_G.T6 = { reqs = {}, clip = {} }
vim.g.clipboard = {
  name = 'termsweep',
  copy = {
    ['+'] = function(lines, rt) _G.T6.clip[#_G.T6.clip+1] = '+:' .. table.concat(lines, '|') .. ':' .. tostring(rt) end,
    ['*'] = function(lines, rt) _G.T6.clip[#_G.T6.clip+1] = '*:' .. table.concat(lines, '|') .. ':' .. tostring(rt) end,
  },
  paste = {
    ['+'] = function() return { { 'PLUS' }, 'v' } end,
    ['*'] = function() return { { 'STAR' }, 'v' } end,
  },
}
vim.api.nvim_create_autocmd('TermRequest', { callback = function(ev)
  local d = ev.data or {}
  _G.T6.reqs[#_G.T6.reqs+1] = tostring(d.sequence) .. '@'
    .. (type(d.cursor) == 'table' and table.concat(d.cursor, ',') or tostring(d.cursor))
    .. '@' .. tostring(d.terminator == '\7' and 'BEL' or 'ST')
end })
function _G.T6NEW()
  _G.TERM_NEW({ rows = 8 })
  _G.T6.reqs, _G.T6.clip = {}, {}
end
function _G.T6WAIT(n)
  if n > 0 then
    vim.wait(4000, function() return #_G.T6.reqs >= n end, 1)
  else
    -- Nothing is expected; one turn of the loop proves it stayed that
    -- way rather than merely not having arrived yet.
    vim.wait(30, function() return #_G.T6.reqs > 0 end, 1)
  end
end
function _G.T6REPLY(n)
  vim.wait(4000, function() return #_G.TS.input >= n end, 1)
end
function _G.T6ANS()
  return {
    reqs = _G.T6.reqs, clip = _G.T6.clip, replies = _G.TS.input,
    title = tostring(vim.b[_G.TS.buf].term_title),
    lines = vim.api.nvim_buf_line_count(_G.TS.buf),
    first = vim.api.nvim_buf_get_lines(_G.TS.buf, 0, 1, false)[1] or '',
  }
end
]==]

-- The `TermRequest` autocommand and the emulator's replies are both
-- delivered by the EVENT LOOP, so every case waits for its own effect.
--
-- THE WHOLE SECTION RUNS IN A CHILD, and that is not tidiness: the
-- sections above leave live terminals behind that have already fed
-- OSCs to the emulator, and the moment a `TermRequest` autocommand
-- exists in the process those arrive on the next turn of the loop --
-- in whichever case happens to wait first.  Two runs of the sweep put
-- them in different cases (`t1/title`'s OSC 0 and `t3/hyperlink`'s two
-- OSC 8s landed in `t6/osc7-cwd` in one run and nowhere in the next).
-- A child that has never run another section cannot have that state.
section('t6-request', function()
  local C = child_new()
  C:lua(T6_PRELUDE)

  local cases = {
    { 'osc7-cwd', '\27]7;file://host/tmp/x\27\\', 1 },
    { 'osc8-hyperlink', '\27]8;;https://example.com\27\\link\27]8;;\27\\', 0 },
    { 'osc9-notify', '\27]9;hello\27\\', 1 },
    { 'osc11-bg-query', '\27]11;?\27\\', 1 },
    { 'osc52-clipboard', '\27]52;c;aGVsbG8=\27\\', 1 },
    -- A clipboard READ is refused by design ("for security reasons").
    { 'osc52-query', '\27]52;c;?\27\\', 1 },
    { 'osc52-primary', '\27]52;p;d29ybGQ=\27\\', 1 },
    { 'osc52-bad-base64', '\27]52;c;!!!!\27\\', 1 },
    { 'osc133-prompt', '\27]133;A\27\\', 1 },
    { 'osc0-title', '\27]0;t-zero\7', 0 },
    { 'osc2-title', '\27]2;t-two\27\\', 0 },
    { 'osc1-icon', '\27]1;t-icon\27\\', 0 },
    { 'dcs-decrqss', '\27P$qm\27\\', 1 },
    { 'apc', '\27_payload\27\\', 1 },
    { 'unknown-osc', '\27]777;notify;x\27\\', 1 },
  }
  for _, c in ipairs(cases) do
    C:lua('_G.T6NEW()')
    C:lua(string.format('_G.TERM_SEND(%q .. "body\\r\\n") _G.T6WAIT(%d)', c[2], c[3]))
    local got = C:lua('return _G.T6ANS()')
    if type(got) ~= 'table' then
      ans('t6/' .. c[1], { got = tostring(got) })
    else
      ans('t6/' .. c[1], {
        reqs = got.reqs or {},
        clip = got.clip or {},
        replies = got.replies or {},
        title = tostring(got.title),
        lines = got.lines,
        first = tostring(got.first),
      })
    end
  end

  -- The reply writer: a query the emulator answers on its own, read
  -- back out of `on_input`.
  for _, c in ipairs({
    { 'da1', '\27[c' },
    { 'da2', '\27[>c' },
    { 'dsr-status', '\27[5n' },
    { 'dsr-cursor', '\27[6n' },
    { 'decrqm', '\27[?1049$p' },
    { 'xtversion', '\27[>0q' },
  }) do
    C:lua('_G.T6NEW()')
    C:lua(string.format('_G.TERM_SEND(%q) _G.T6REPLY(1)', c[2]))
    local got = C:lua('return _G.TS.input')
    ans('t6/reply/' .. c[1], { replies = type(got) == 'table' and got or { tostring(got) } })
  end

  -- `terminal_notify_theme`: only a terminal that asked (DECSET 2031)
  -- is told that `'background'` changed.
  for _, want in ipairs({ false, true }) do
    C:lua('_G.T6NEW()')
    if want then
      C:lua('_G.TERM_SEND("\\27[?2031h")')
    end
    C:lua([[
      _G.TS.input = {}
      vim.o.background = 'light'
      vim.o.background = 'dark'
      vim.wait(200, function() return #_G.TS.input >= 2 end, 1)
    ]])
    local got = C:lua('return _G.TS.input')
    ans('t6/theme/' .. tostring(want), { replies = type(got) == 'table' and got or { tostring(got) } })
  end

  -- A `TermRequest` handler that writes back: the send is HELD while a
  -- handler runs (`TerminalPending::send`) and flushed after it.
  C:lua('_G.T6NEW()')
  C:lua([[
    _G.HELD = vim.api.nvim_create_autocmd('TermRequest', { buffer = _G.TS.buf, callback = function(ev)
      if tostring((ev.data or {}).sequence):match('133') then
        vim.api.nvim_chan_send(_G.TS.chan, 'from-handler\r\n')
      end
    end })
    _G.TERM_SEND('\27]133;B\27\\tail\r\n')
    -- Both effects, each by its own condition: the sequence has to be
    -- reported (the autocommand ran) and the handler's own write has to
    -- have reached the buffer.  The second is bounded separately so
    -- that "it never arrives" is a stable answer rather than a race.
    vim.wait(4000, function() return #_G.T6.reqs > 0 end, 1)
    vim.wait(2000, function()
      _G.TERM_POKE()
      return table.concat(vim.api.nvim_buf_get_lines(_G.TS.buf, 0, -1, false), '\n'):find('from%-handler') ~= nil
    end, 5)
    _G.TERM_POKE()
  ]])
  local held = C:lua([[
    local l = vim.api.nvim_buf_get_lines(_G.TS.buf, 0, -1, false)
    local n = {}
    for _, x in ipairs(l) do if x ~= '' then n[#n+1] = x end end
    return { lines = #l, text = n, reqs = _G.T6.reqs }
  ]])
  if type(held) == 'table' then
    ans('t6/held-send', { lines = held.lines, text = held.text or {}, reqs = held.reqs or {} })
  else
    ans('t6/held-send', { got = tostring(held) })
  end

  ans('t6/said', { stderr = C:said() })
  C:stop()
end)

-- =============================================================== t7
-- The real pty path.  This is the code the deterministic drive above
-- deliberately does NOT reach: `terminal_open`'s job, the process
-- exit, the `[Process exited]` extmark, TermOpen/TermClose, and
-- `:terminal` itself.  Every case syncs on an EFFECT -- `jobwait` for
-- the exit, `vim.wait` on the line that proves the output arrived.
-- ====================================================================

section('t7-pty', function()
  vim.o.shell = '/bin/sh'

  --- Run one `:terminal`, wait for the child to exit AND for the
  --- refresh that carries its last line, then answer.
  local function run(label, cmd, opts)
    opts = opts or {}
    pcall(vim.cmd, 'silent! only!')
    vim.o.lines = 24
    vim.o.columns = 80
    vim.o.laststatus = 0
    local events = {}
    local ids = {}
    for _, ev in ipairs({ 'TermOpen', 'TermClose', 'TermEnter', 'TermLeave' }) do
      ids[#ids + 1] = vim.api.nvim_create_autocmd(ev, {
        callback = function(a)
          events[#events + 1] = ev .. ':' .. tostring(vim.bo[a.buf].buftype)
        end,
      })
    end
    vim.cmd('enew')
    local buf, jid
    if opts.api then
      buf = vim.api.nvim_create_buf(false, true)
      vim.api.nvim_win_set_buf(0, buf)
      vim.api.nvim_buf_call(buf, function()
        jid = vim.fn.jobstart({ '/bin/sh', '-c', cmd }, { term = true })
      end)
    else
      vim.cmd('terminal /bin/sh -c ' .. vim.fn.shellescape(cmd))
      buf = vim.api.nvim_get_current_buf()
      jid = vim.bo[buf].channel
    end
    local rc = vim.fn.jobwait({ jid }, 20000)[1]
    -- The exit is not the end: `TermClose` and the last refresh are
    -- delivered by the event loop.
    until_(function()
      return #events >= 2
    end, 5000)
    if opts.want then
      until_(function()
        return table.concat(vim.api.nvim_buf_get_lines(buf, 0, -1, false), '\n'):find(opts.want, 1, true) ~= nil
      end, 5000)
    end
    vim.cmd('redraw')
    local marks = vim.api.nvim_buf_get_extmarks(buf, -1, 0, -1, { details = true })
    local mtxt = {}
    for _, m in ipairs(marks) do
      local d = m[4] or {}
      local vt = {}
      for _, chunk in ipairs(d.virt_text or {}) do
        vt[#vt + 1] = tostring(chunk[1])
      end
      mtxt[#mtxt + 1] = string.format('r%d:%s', m[2], table.concat(vt, '|'))
    end
    local lines = vim.api.nvim_buf_get_lines(buf, 0, -1, false)
    local nonempty = {}
    for _, l in ipairs(lines) do
      if l ~= '' then
        nonempty[#nonempty + 1] = l
      end
    end
    local a = {
      rc = rc,
      events = events,
      extmarks = mtxt,
      lines = #lines,
      text = nonempty,
      -- Left whole so that `scrub` can take the pid out of
      -- `term://<cwd>//<pid>:<cmd>`; stripping the prefix here first
      -- would leave the pid behind and re-baseline on every run.
      name = vim.api.nvim_buf_get_name(buf),
      title = tostring(vim.b[buf].term_title),
      buftype = vim.bo[buf].buftype,
      modified = tostring(vim.bo[buf].modified),
      job_id = tostring(vim.b[buf].terminal_job_id ~= nil),
      job_pid = tostring(vim.b[buf].terminal_job_pid ~= nil),
      running = tostring(vim.bo[buf].channel ~= 0),
    }
    for _, id in ipairs(ids) do
      pcall(vim.api.nvim_del_autocmd, id)
    end
    ans(label, a)
    return buf
  end

  run('t7/echo', 'printf "one\\ntwo\\nthree\\n"', { want = 'three' })
  run('t7/exit7', 'exit 7', {})
  run('t7/overflow', 'i=1; while [ $i -le 60 ]; do echo "p $i"; i=$((i+1)); done', { want = 'p 60' })
  run('t7/sgr', 'printf "\\033[31mred\\033[0m done\\n"', { want = 'done' })
  run('t7/title', 'printf "\\033]0;pty-title\\007body\\n"', { want = 'body' })
  run('t7/api-jobstart', 'printf "api\\n"', { api = true, want = 'api' })

  -- `:terminal` with no argument runs `'shell'`; the shell reads EOF
  -- from the closed stdin of a headless nvim and exits.
  run('t7/bare-shell', 'exit 0', {})

  -- wipe: the buffer goes away under a LIVE job.  `terminal_destroy`
  -- and the close callback must both survive it.
  pcall(vim.cmd, 'silent! only!')
  vim.cmd('enew')
  vim.cmd('terminal /bin/sh -c "sleep 30"')
  local wbuf = vim.api.nvim_get_current_buf()
  local wjid = vim.bo[wbuf].channel
  local alive = vim.api.nvim_buf_is_valid(wbuf)
  vim.cmd('bwipeout!')
  until_(function()
    return vim.fn.jobwait({ wjid }, 0)[1] ~= -1
  end, 5000)
  vim.cmd('redraw')
  ans('t7/wipe', {
    alive_before = tostring(alive),
    valid_after = tostring(vim.api.nvim_buf_is_valid(wbuf)),
    job_after = vim.fn.jobwait({ wjid }, 0)[1],
    wins = #vim.api.nvim_list_wins(),
  })

  -- The job killed under a live terminal, then the buffer left open:
  -- the `[Process exited]` extmark is placed by the close path.
  pcall(vim.cmd, 'silent! only!')
  vim.cmd('enew')
  vim.cmd('terminal /bin/sh -c "sleep 30"')
  local kbuf = vim.api.nvim_get_current_buf()
  local kjid = vim.bo[kbuf].channel
  vim.fn.jobstop(kjid)
  vim.fn.jobwait({ kjid }, 10000)
  until_(function()
    return vim.bo[kbuf].channel == 0
  end, 5000)
  vim.cmd('redraw')
  local km = {}
  for _, m in ipairs(vim.api.nvim_buf_get_extmarks(kbuf, -1, 0, -1, { details = true })) do
    local vt = {}
    for _, chunk in ipairs((m[4] or {}).virt_text or {}) do
      vt[#vt + 1] = tostring(chunk[1])
    end
    km[#km + 1] = string.format('r%d:%s', m[2], table.concat(vt, '|'))
  end
  ans('t7/killed', {
    extmarks = km,
    running = tostring(vim.bo[kbuf].channel ~= 0),
    modified = tostring(vim.bo[kbuf].modified),
  })
  pcall(vim.cmd, 'bwipeout!')
end)

-- =============================================================== t91
-- The inputs that may kill the editor, one FRESH CHILD each, so that a
-- death costs one row instead of the rest of the sweep.
--
-- `t91/onecol-wide` USED TO BE THE DOCUMENTED ABORT: a terminal ONE
-- COLUMN wide fed a double-width glyph died in `putglyph`'s
-- continuation loop, which dereferenced the NULL `getcell` answers past
-- the last column.  `v0.12.4`'s `src/nvim/vterm/screen.c` has the
-- identical unguarded write, so upstream segfaults there too; it is
-- filed as `vterm-screen-putglyph-narrow-null-deref.md`.  The port
-- fixed it at B21-9 (`c613f92ab5`) and the row was re-baselined
-- deliberately; the three kept sides still abort on it, which is the
-- exception set `termverify.sh` documents.
--
-- `t91/palette-unset` WAS the other one, and it was OURS: the port's
-- `get_config_string` freed the `g:terminal_color_N` string that
-- `dict_get_value`'s `reuse_strdata` had made a BORROW of the
-- variable's own bytes, so unsetting or reassigning the variable freed
-- it twice.  Upstream never freed it.  Fixed at B21-10 (`d139fba127`)
-- and re-baselined the same way, which is why the exception set has
-- three rows rather than two.
--
-- THE BASELINE IS NOW `aborted=0`: no probe here is expected to die,
-- and ANY abort is a regression.
-- ====================================================================

section('t91-abortprobe', function()
  local probes = {
    { 'plain', [[_G.TERM_NEW({rows=8}) _G.TERM_SEND('hello\r\n')]] },
    -- The documented one.
    {
      'onecol-wide',
      [[_G.TERM_NEW({rows=8, cols=1}) _G.TERM_SEND('\228\184\173')]],
    },
    { 'onecol-narrow', [[_G.TERM_NEW({rows=8, cols=1}) _G.TERM_SEND('abc\r\n')]] },
    { 'twocol-wide', [[_G.TERM_NEW({rows=8, cols=2}) _G.TERM_SEND('\228\184\173\228\184\173')]] },
    { 'onerow', [[_G.TERM_NEW({rows=1}) _G.TERM_SEND('a\r\nb\r\nc\r\n')]] },
    { 'huge-csi-params', [[_G.TERM_NEW({rows=8}) _G.TERM_SEND('\27[' .. string.rep('1;', 200) .. '1m x')]] },
    { 'rep-huge', [[_G.TERM_NEW({rows=8}) _G.TERM_SEND('x\27[60000b')]] },
    { 'decstbm-inverted', [[_G.TERM_NEW({rows=8}) _G.TERM_SEND('\27[20;2r\27[5;1Hx\r\n')]] },
    { 'sb-zero', [[_G.TERM_NEW({rows=8}) vim.bo[_G.TS.buf].scrollback = 1 for i=1,50 do _G.TERM_SEND('z ' .. i .. '\r\n') end]] },
    { 'osc52-huge', [[_G.TERM_NEW({rows=8}) _G.TERM_SEND('\27]52;c;' .. string.rep('QUJD', 5000) .. '\27\\')]] },
    { 'dcs-unterminated', [[_G.TERM_NEW({rows=8}) _G.TERM_SEND('\27P' .. string.rep('q', 5000))]] },
    { 'altscreen-storm', [[_G.TERM_NEW({rows=6}) for i=1,40 do _G.TERM_SEND('\27[?1049h a ' .. i .. '\r\n\27[?1049l b\r\n') end]] },
    { 'resize-storm', [[_G.TERM_NEW({rows=10}) _G.TERM_SEND(string.rep('R', 300)) for h=2,20,3 do vim.api.nvim_win_set_height(0, h) vim.cmd('redraw') _G.TERM_POKE() end]] },
    { 'wipe-live', [[_G.TERM_NEW({rows=8}) _G.TERM_SEND('x\r\n') vim.cmd('bwipeout!')]] },
    { 'close-in-request', [[
        _G.TERM_NEW({rows=8})
        vim.api.nvim_create_autocmd('TermRequest', { callback = function() pcall(vim.cmd, 'bwipeout!') end })
        _G.TERM_SEND('\27]133;A\27\\x\r\n')
        vim.wait(200)
      ]] },
    -- The port-side double free above, from both directions.
    { 'palette-unset', [[
        vim.g.terminal_color_1 = '#00ff88'
        _G.TERM_NEW({rows=8})
        _G.TERM_SEND('\27[31mx\27[0m\r\n')
        vim.g.terminal_color_1 = nil
      ]] },
    { 'palette-reset', [[
        vim.g.terminal_color_2 = '#00ff88'
        _G.TERM_NEW({rows=8})
        _G.TERM_SEND('\27[32mx\27[0m\r\n')
        vim.g.terminal_color_2 = '#112233'
      ]] },
    { 'palette-read', [[
        vim.g.terminal_color_3 = '#00ff88'
        _G.TERM_NEW({rows=8})
        _G.TERM_SEND('\27[33mx\27[0m\r\n')
        _G.READBACK = tostring(vim.g.terminal_color_3)
      ]] },
    { 'send-after-close', [[
        _G.TERM_NEW({rows=8})
        local ch = _G.TS.chan
        vim.fn.chanclose(ch)
        pcall(vim.api.nvim_chan_send, ch, 'after\r\n')
      ]] },
  }
  local aborted = 0
  for _, p in ipairs(probes) do
    local C = child_new()
    local res = C:lua(p[2] .. ' return "ok"')
    local alive = C:lua('return 1') == 1
    ans('t91/' .. p[1], {
      res = tostring(res),
      alive = tostring(alive),
      said = C:said(),
    })
    if not alive then
      aborted = aborted + 1
    end
    C:stop()
  end
  ans('t91/groups', { cases = #probes, aborted = aborted })
end)

emit('##', 'TOTAL', string.format('rows=%d', rows))
structfd:close()
