-- bufsweep -- the NINETEENTH baselined differential.  Driven by
-- bufsweep.sh, which builds the sandbox, pins $HOME /
-- $TMPDIR / $PATH and does the scrubs only the shell can see.  Read
-- that header first.
--
-- The subsystem is buffer.rs (4,960 lines, 4,661 unchecked): the whole
-- buffer-list family -- `buflist_list` (the `:ls` renderer, 200 lines),
-- `chk_modeline` (180), `ExpandBufnames` (181), `fileinfo` (157),
-- `buflist_findpat` (133), `buflist_new`, `buflist_findname*`,
-- `buflist_setfpos`/`buflist_findfmark`, `do_buffer_ext`,
-- `close_buffer`, `open_buffer`, `enter_buffer`, `setfname`/
-- `buf_name_changed`, `bt_*`/`buf_spname` and `ex_buffer_all`.
--
-- The gap this closes: B20's survey S5, hole 3.  Of EIGHTEEN baselined
-- differentials, `:ls`/`:buffers` appeared only as a *name* in the ex
-- probe and as a *parse target* in cmdsweep -- no oracle anywhere read
-- its output; `chk_modeline` was reached only incidentally, by
-- fmtsweep's and stlsweep's `'formatoptions'` fixtures; `CTRL-G` was in
-- five bench/probe files and no differential; and `ExpandBufnames` and
-- `buflist_findpat` were untested.  About 850 lines of PURE STRING
-- BEHAVIOUR with no differential at all.
--
-- THE DESIGN, in one paragraph.  Everything this family does is a
-- string (`:ls`'s six flag columns and its 40-column pad, `fileinfo`'s
-- message, an error number) or a small number (a bufnr, a triple of
-- `bufexists`/`buflisted`/`bufloaded`), so the sweep is textual and the
-- bulk of it runs in ONE `--headless` process, exactly as stlsweep,
-- menusweep and winsweep do.  It is NOT `-l` (b20-2's rule: `-l` sets
-- `silent_mode`, `full_screen` is `!silent_mode`, and whole arms of an
-- oracle vanish silently under it) and it runs from `VimEnter`, not
-- from `-c`.
--
-- BUFFER NUMBERS ARE PRINTED RAW, AND THAT IS DELIBERATE.  winsweep
-- renumbers every window handle to an ordinal because a window id is
-- an invisible process-global counter; a BUFFER number is not -- `:ls`
-- prints it in its first column, `CTRL-G`'s `2CTRL-G` form prints it,
-- `bufnr()` returns it, and a sweep that hid it would stop gating the
-- one thing users read.  Determinism is bought a different way:
--
--   * the in-process sections (b0, b1, b2, b4, b5, b6, b7, b8) share
--     ONE fixture, built once before any section runs, and NONE of
--     them may create or destroy a buffer.  Each section's `##` line
--     carries `nbuf=` and `lastbuf=` so a section that leaks one is a
--     one-line diff instead of a silent renumbering of everything
--     below it.
--   * every section that DOES churn the buffer list -- b3 lifecycle,
--     b9 autocmd order, b91 crashprobe -- runs one FRESH CHILD PROCESS
--     PER CASE.  A child starts at bufnr 1 every time, so an inserted
--     case renumbers nothing, and a case that kills the editor takes
--     only itself with it.
--
-- `b_last_used` IS A WALL CLOCK, and `:ls t` sorts on it.  Two buffers
-- entered in the same second TIE, and `buf_time_compare` answers 0 --
-- so whether they straddle a second boundary decides the order, which
-- is a one-in-twenty flake in a baselined artifact.  Every fixture step
-- that ENTERS a buffer is therefore separated by `tick()`, a spin until
-- `os.time()` changes: the four entered buffers have four strictly
-- ordered timestamps and everything else has 0.  The rendered time is
-- scrubbed (`undo_fmt_time` prints "N seconds ago" under 100 s and
-- `%H:%M:%S` over it), so what the artifact keeps is the ORDER.
--
-- Sections:
--   b0  defaults    the family's option defaults and the fixture roster
--   b1  lsflags     `:ls`/`:files`/`:buffers` x every flag and `!`
--   b2  address     `:buffer N|#|name`, `:bnext`/`:bprev`/`:bfirst`/
--                   `:blast` with counts and wrap, `:bmodified`,
--                   `:badd +lnum`, `:balt`, CTRL-^, `:bufdo`
--   b3  lifecycle   `:bdelete`/`:bwipeout`/`:bunload` -- ONE CHILD EACH
--   b4  findpat     `bufnr()`/`bufname()` through `buflist_findpat`
--   b5  complete    `getcompletion(..., 'buffer')` -- `ExpandBufnames`
--   b6  fileinfo    `:file`, CTRL-G, 1/2CTRL-G, g CTRL-G, 'shortmess'
--   b7  modeline    the `chk_modeline` corpus
--   b8  bufinfo     `getbufinfo()` dicts, `get/setbufvar`, the triples
--   b9  auorder     Buf* autocmd ORDER -- ONE CHILD EACH
--   b91 crashprobe  the inputs that may kill the editor -- ONE CHILD EACH
--
-- Every section ends with a `## <name> rows=N nbuf=N lastbuf=N` line.
-- A section that goes silently empty otherwise looks exactly like a
-- healthy one, and a section that leaks a buffer looks exactly like a
-- behaviour change three sections later.

local work = assert(os.getenv('BUF_WORK'), 'BUF_WORK unset')
local runtime = os.getenv('VIMRUNTIME') or ''
local script = debug.getinfo(1, 'S').source:sub(2)

local only = os.getenv('BUFSWEEP_ONLY')
if only == '' then
  only = nil
end
local trace = os.getenv('BUFSWEEP_TRACE') == '1'

io.stdout:setvbuf('line')

local rows = 0
local function emit(...)
  rows = rows + 1
  io.write(table.concat({ ... }, ' '), '\n')
end

-- --------------------------------------------------------------- scrub

--- $WORK first: $HOME and $TMPDIR are inside it.  The two time forms
--- are `undo_fmt_time`'s, reached from `:ls t` and from nothing else.
local function scrub(text)
  text = tostring(text)
  text = text:gsub(vim.pesc(work), '<WORK>')
  text = text:gsub(vim.pesc(script), '<SCRIPT>')
  if runtime ~= '' then
    text = text:gsub(vim.pesc(runtime), '<RT>')
  end
  text = text:gsub('%d%d%d%d/%d%d/%d%d %d%d:%d%d:%d%d', '<DATE>')
  text = text:gsub('%d%d:%d%d:%d%d', '<TIME>')
  text = text:gsub('%-?%d+ seconds? ago', '<AGO>')
  text = text:gsub('nvim%.%d+%.%d+', 'nvim.<PID>.<SEQ>')
  -- `fileinfo`'s non-`dont_truncate` arm runs the whole message
  -- through `msg_trunc`, which cuts the HEAD off and marks it with a
  -- `<`.  A truncated $VIMRUNTIME no longer starts with $VIMRUNTIME,
  -- so the substitution above cannot see it and the artifact would
  -- carry this machine's absolute repo path.  The `<` is kept: whether
  -- the message truncates at all is behaviour.
  text = text:gsub('/[%w_.@%-/]*/runtime/doc/', '<RT~>/doc/')
  return text
end

local function cap(text, limit)
  limit = limit or 1200
  if #text <= limit then
    return text
  end
  return text:sub(1, limit) .. string.format('...<+%d>', #text - limit)
end

--- Escape to one printable line.  Buffer names are deliberately
--- multibyte in places and a raw byte would make `diff` call the
--- report binary and print nothing useful.
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
  assert(io.open(assert(os.getenv('BUF_STRUCT'), 'BUF_STRUCT unset'), 'w'))

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
    local v = value[k]
    if v == nil then
      v = value[tonumber(k)]
    end
    parts[#parts + 1] = q(k) .. ':' .. canon(v)
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
  s = s:gsub('^[^\n]-bufsweep%.lua:%d+: ', '')
  s = s:gsub('^%[string "[^"]*"%]:%d+: ', '')
  s = s:gsub('^nvim_exec2%(%), line %d+: ', '')
  -- The sweep runs inside a `VimEnter` autocmd, so every error raised
  -- by `nvim_exec2` arrives wrapped in that context.  The wrapper is a
  -- fact about the harness, not about the editor.
  s = s:gsub('^VimEnter Autocommands.-, line %d+: ', '')
  s = s:gsub('\r?\n', ' | ')
  return cap(scrub(s))
end

-- ------------------------------------------------------------ sections

local secrows = 0
local function nbuf()
  local n, last = 0, 0
  for _, b in ipairs(vim.api.nvim_list_bufs()) do
    n = n + 1
    if b > last then
      last = b
    end
  end
  return n, last
end

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
  local n, last = nbuf()
  emit(
    '##',
    name,
    string.format('rows=%d nbuf=%d lastbuf=%d', rows - secrows - 1, n, last)
  )
end

-- ============================================================ the world

--- The option baseline.  Every case starts from exactly this.
---
--- `'hidden'` is pinned ON: without it `:bnext` off a modified buffer
--- is E37 and half of b2 measures the guard instead of the walk.
--- `'modeline'` is pinned OFF here and turned on only inside b7, so an
--- upstream default change shows up in b0 as one row.
local BASEOPT = {
  'set noswapfile nobackup nowritebackup hidden',
  'set report=9999 shortmess=aoOtTIcCF nomore belloff=all',
  'set lines=24 columns=80 cmdheight=1 laststatus=2 showtabline=1',
  'set nomodeline modelines=5 nomodelineexpr',
  'set noruler noshowcmd nonumber norelativenumber nowrap',
  'set noequalalways nosplitbelow nosplitright winminheight=0',
  'set switchbuf= noautowrite noautowriteall noautoread',
  'set nofileignorecase nowildignorecase wildoptions=',
  'set eventignore= verbose=0 magic',
  'set tabstop=8 shiftwidth=8 noexpandtab textwidth=0 noautoindent',
  'set nolist nospell foldmethod=manual commentstring=',
}

local function baseopts()
  for _, line in ipairs(BASEOPT) do
    pcall(vim.api.nvim_exec2, line, { output = false })
  end
end

--- Spin until the wall clock ticks.  `b_last_used` is `time()` and
--- `buf_time_compare` answers 0 for a tie, so two buffers entered in
--- the same second sort in an order glibc's qsort decides -- and
--- whether they tie at all depends on where the run lands inside a
--- second.  Every ENTERING fixture step is bracketed by this, which
--- costs four seconds once and buys a `:ls t` order that is a fact
--- about `buf_time_compare` rather than about the clock.
local function tick()
  local t0 = os.time()
  while os.time() == t0 do
    -- busy: os.time() has one-second resolution and vim.wait would
    -- turn the event loop, which is not wanted mid-fixture.
  end
end

-- --------------------------------------------------------------- files

local function W(rel)
  return work .. '/' .. rel
end

local function writefile(rel, lines)
  vim.fn.writefile(lines, W(rel))
end

-- ------------------------------------------------------------- answers

--- `:ls` and friends: the OUTPUT IS THE ARTIFACT.  Newlines become
--- ` | ` so one case is one line and `diff` stays readable.
local function lsout(cmd)
  local ok, res = pcall(vim.api.nvim_exec2, cmd, { output = true })
  if not ok then
    return '!! ' .. errtext(res)
  end
  local o = res.output or ''
  o = o:gsub('^\n', '')
  if o == '' then
    return '<empty>'
  end
  return (o:gsub('\r?\n', ' | '))
end

--- The structured half of every lifecycle answer: for each of the
--- fixture's buffer numbers, `bufexists`/`buflisted`/`bufloaded` as a
--- triple, plus the current and alternate numbers.  `:ls` cannot see a
--- wiped buffer at all, and the difference between "unlisted" and
--- "gone" is exactly what `:bdelete` and `:bwipeout` disagree about.
local function triples(upto)
  local t = {}
  for n = 1, (upto or 24) do
    if vim.fn.bufexists(n) == 1 then
      t[#t + 1] = string.format(
        '%d:%d%d%d',
        n,
        vim.fn.bufexists(n),
        vim.fn.buflisted(n),
        vim.fn.bufloaded(n)
      )
    end
  end
  return table.concat(t, ' ')
end

local function curalt()
  return string.format(
    'cur=%d alt=%d name=%s',
    vim.fn.bufnr('%'),
    vim.fn.bufnr('#'),
    vim.fn.bufname('%')
  )
end

local function ans(label, note, text, value)
  label_once(label)
  emit(label, '=', esc(scrub(cap(tostring(note), 400))), esc(scrub(cap(tostring(text)))))
  struct(label, value or { note = tostring(note), text = tostring(text) })
end

--- Run commands, capture each one's own output or error, then answer.
--- `extra` may be a function: it is called AFTER the commands run, so
--- a case can answer with something the commands changed (the window
--- count, the remembered cursor line).  A plain string is evaluated at
--- the call site and would answer the state BEFORE the case.
local function run(label, cmds, extra)
  local outs = {}
  for _, c in ipairs(type(cmds) == 'table' and cmds or { cmds }) do
    local ok, res = pcall(vim.api.nvim_exec2, c, { output = true })
    if ok then
      local o = (res.output or ''):gsub('^\n', ''):gsub('\r?\n', ' | ')
      outs[#outs + 1] = c .. (o == '' and '' or (' -> ' .. o))
    else
      outs[#outs + 1] = c .. ' !! ' .. errtext(res)
    end
  end
  local note = table.concat(outs, ' ;; ')
  if type(extra) == 'function' then
    local ok, v = pcall(extra)
    extra = ok and tostring(v) or ('EXTRA-ERR ' .. errtext(v))
  end
  local body = curalt() .. ' ' .. (extra or '')
  label_once(label)
  emit(label, '=', esc(scrub(cap(note, 500))), esc(scrub(cap(body))))
  struct(label, {
    cmds = type(cmds) == 'table' and cmds or { cmds },
    note = note,
    cur = vim.fn.bufnr('%'),
    alt = vim.fn.bufnr('#'),
    name = vim.fn.bufname('%'),
    body = body,
  })
  return note
end

-- ================================================================ child
-- b3, b9 and b91 only.  A fresh process per case is what makes a raw
-- bufnr safe to print: every child starts at 1, so an inserted case
-- renumbers nothing below it, and a case that aborts takes only itself.
-- ====================================================================

local CHILD_ENV = {
  HOME = work .. '/home',
  PATH = work .. '/bin',
  TMPDIR = work .. '/tmp',
  TERM = 'dumb',
  SHELL = '/bin/sh',
  LANG = 'C.UTF-8',
  VIMRUNTIME = runtime,
  NVIM_TEST = '1',
  BUF_WORK = work,
}

--- The child's own prelude: the same option baseline, a small
--- deterministic fixture and the two answer helpers.  It is a STRING
--- because the child has no access to this file.
local CPRELUDE = [[
local WK = vim.env.BUF_WORK
_G.OPTS = ]] .. vim.inspect(BASEOPT):gsub('%s+', ' ') .. [[

function _G.BASE()
  for _, c in ipairs(_G.OPTS) do pcall(vim.api.nvim_exec2, c, {output=false}) end
end
--- Buffers 2..7, always the same numbers: two plain files, one
--- modified, one listed-but-never-loaded, one unlisted scratch and one
--- nomodifiable.  Buffer 1 is the startup [No Name].
function _G.FIX()
  local b2 = vim.fn.bufadd('one.txt');   vim.fn.bufload(b2); vim.bo[b2].buflisted = true
  local b3 = vim.fn.bufadd('two.txt');   vim.fn.bufload(b3); vim.bo[b3].buflisted = true
  local b4 = vim.fn.bufadd('three.txt'); vim.fn.bufload(b4); vim.bo[b4].buflisted = true
  vim.api.nvim_buf_set_lines(b4, 0, -1, false, {'three CHANGED'})
  local b5 = vim.fn.bufadd('four.txt');  vim.bo[b5].buflisted = true
  local b6 = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_name(b6, 'scratch-unlisted')
  local b7 = vim.fn.bufadd('five.txt');  vim.fn.bufload(b7); vim.bo[b7].buflisted = true
  vim.bo[b7].modifiable = false
  return {b2, b3, b4, b5, b6, b7}
end
function _G.LS(flags)
  local ok, r = pcall(vim.api.nvim_exec2, 'ls' .. (flags or ''), {output=true})
  if not ok then return '!! ' .. tostring(r) end
  local o = (r.output or ''):gsub('^\n', '')
  if o == '' then return '<empty>' end
  return (o:gsub('\r?\n', ' | '))
end
function _G.TRIP(upto)
  local t = {}
  for n = 1, (upto or 16) do
    if vim.fn.bufexists(n) == 1 then
      t[#t+1] = string.format('%d:%d%d%d', n, vim.fn.bufexists(n), vim.fn.buflisted(n), vim.fn.bufloaded(n))
    end
  end
  return table.concat(t, ' ')
end
function _G.CUR()
  return string.format('cur=%d alt=%d name=%s nwin=%d',
    vim.fn.bufnr('%'), vim.fn.bufnr('#'), vim.fn.bufname('%'), vim.fn.winnr('$'))
end
function _G.DO(cmds)
  local outs = {}
  for _, c in ipairs(cmds) do
    local ok, r = pcall(vim.api.nvim_exec2, c, {output=true})
    if ok then
      local o = (r.output or ''):gsub('^\n', ''):gsub('\r?\n', ' | ')
      outs[#outs+1] = c .. (o == '' and '' or (' -> ' .. o))
    else
      -- The child runs its case inside a `VimEnter` autocmd too, so
      -- every `nvim_exec2` error arrives wrapped in that context.
      local m = (tostring(r):gsub('\r?\n', ' | '))
      m = (m:gsub('^VimEnter Autocommands.-, line %d+: ', ''))
      outs[#outs+1] = c .. ' !! ' .. m
    end
  end
  return table.concat(outs, ' ;; ')
end
--- The child's answer goes to a FILE, never to stdout.  A case that
--- drives the COMMAND LINE (b5's `'wildmode'` block) makes the editor
--- redraw the cmdline, and in `--headless` that redraw is written to
--- stdout with no newline anywhere -- it would glue itself to the
--- front of the answer and there is no prefix filter that survives it.
function _G.OUT(...)
  local line = table.concat({...}, '\t')
  local f = io.open(vim.env.BUF_ANS, 'a')
  f:write(line, '\n')
  f:close()
end
]]

--- One child process.  Returns rc, stdout lines, stderr lines.
local ANS = work .. '/tmp/ans.txt'

local function child(code)
  local err = {}
  os.remove(ANS)
  local env = vim.deepcopy(CHILD_ENV)
  env.BUF_ANS = ANS
  env.BUFCASE = CPRELUDE .. '\n' .. code
  local job = vim.fn.jobstart({
    work .. '/bin/nvim',
    '--headless',
    '-u',
    'NONE',
    '-i',
    'NONE',
    '--cmd',
    'set noswapfile nobackup nowritebackup',
    '--cmd',
    -- `++nested`: an autocommand fired while another autocommand runs
    -- is SILENTLY SKIPPED without it, and the whole b9 section then
    -- answers an empty sequence for every gesture.
    'autocmd VimEnter * ++once ++nested lua local f, e = load(vim.env.BUFCASE, "case") '
      .. 'if not f then io.stderr:write("CASE-COMPILE " .. tostring(e) .. "\\n") '
      .. 'else local ok, r = pcall(f) if not ok then io.stderr:write("CASE-ERR " .. tostring(r) .. "\\n") end end '
      .. 'vim.cmd("qa!")',
  }, {
    cwd = work,
    clear_env = true,
    env = env,
    stdout_buffered = true,
    stderr_buffered = true,
    on_stdout = function() end,
    on_stderr = function(_, data)
      err = data or {}
    end,
  })
  if job <= 0 then
    return -1, {}, { 'JOBSTART FAILED' }
  end
  local rc = vim.fn.jobwait({ job }, 60000)[1]
  local out = vim.fn.filereadable(ANS) == 1 and vim.fn.readfile(ANS) or {}
  local function clean(list)
    local t = {}
    for _, line in ipairs(list) do
      line = line:gsub('\r$', '')
      if line ~= '' then
        t[#t + 1] = line
      end
    end
    return t
  end
  return rc, clean(out), clean(err)
end

--- A dying child's stderr, reduced to the part that is a FACT about
--- the editor rather than about this machine.
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

--- One isolated case: run `code` in a fresh editor, print what it
--- printed.  `rc` is part of the answer -- 0 is alive, anything else is
--- the crashprobe verdict.
--- `quiet` keeps only the part of a child's stderr that is a FAULT.
--- b5's `'wildmode'` cases drive a real command line, and in
--- `--headless` the cmdline REDRAW is written to stderr -- twenty rows
--- of `:buffer one.txt:buffer two.txt...` that would re-baseline on any
--- change to cmdline drawing, a subsystem this oracle does not gate.
local function faults(lines)
  local out = {}
  for _, line in ipairs(lines or {}) do
    if
      line:match('panic')
      or line:match('assertion')
      or line:match('RUST_BACKTRACE')
      or line:match('SIG%u')
      or line:match('CASE%-')
    then
      out[#out + 1] = line
    end
  end
  return out
end

local function case(label, code, quiet)
  local rc, out, err = child(code)
  local words = quiet and faults(said(err)) or said(err)
  label_once(label)
  emit(
    label,
    '=',
    'rc=' .. tostring(rc),
    esc(scrub(cap(table.concat(out, ' ;; '), 1600))),
    'said=' .. esc(cap(table.concat(words, ' | '), 300))
  )
  struct(label, { rc = rc, out = out, said = words })
  return rc
end

-- ============================================================= fixture
-- Built ONCE, before any section, and never touched again by an
-- in-process section.  Every `##` line carries `nbuf=`/`lastbuf=`, so a
-- section that creates one is a one-line diff rather than a silent
-- renumbering of everything below.
--
--   1  [No Name]      the startup buffer, listed, loaded, NEVER entered
--   2  alpha.txt      plain file, listed, loaded, current at the end
--   3  beta.txt       listed, loaded, MODIFIED           -> `+`
--   4  gamma.txt      listed, loaded then unloaded       -> `h` cleared
--   5  delta.txt      listed, NEVER loaded (`:badd`)     -> ` `
--   6  epsilon.txt    listed, loaded, 'readonly'         -> `=`
--   7  zeta.txt       listed, loaded, 'nomodifiable'     -> `-`
--   8  unlisted.txt   loaded, NOT listed                 -> `u`
--   9  term-run       terminal with a live channel       -> `R`
--  10  term-done      terminal whose channel is closed   -> `F`
--  11  [Prompt]       'buftype'=prompt, no name
--  12  [Scratch]      'buftype'=nofile, no name
--  13  sub1/dup.txt   duplicate tail, first
--  14  sub2/dup.txt   duplicate tail, second
--  15  h\xc3\xa9llo-\xce\xb4.txt   multibyte name
--  16  two words.txt  a space in the name
--  17  deep/.../long.txt   a long relative path
--  18  ~/inhome.txt   under $HOME, so `home_replace` prints `~/`
--  19  <WORK>/abs.txt opened by ABSOLUTE path
--  20  modeline.txt   b7's one file, re-`:edit!`ed per case (tick 3)
--  21  twin.txt       a plain listed twin of alpha
--  22  [Quickfix List]  unlisted, 'buftype'=quickfix (tick 1)
--  23  helphelp.txt   the help buffer, unlisted, 'readonly' (tick 2)
--
-- Buffer 2 is entered last (tick 4) and left current, so `%` is 2 and
-- `#` is 3.  Buffer 1 is never entered, which is what keeps it a
-- `[No Name]`: `curbuf_reusable()` would hand `:edit` the empty
-- unnamed buffer and b6 would have no `[No Name]` to ask about.
-- FOUR buffers have a non-zero `b_last_used` (22, 23, 20, 2, in that
-- order); every other one has 0, which is what makes `:ls t` a fact
-- about `buf_time_compare` and not about the clock.

local FIX = {}

local function fixture()
  baseopts()
  vim.fn.mkdir(W('sub1'), 'p')
  vim.fn.mkdir(W('sub2'), 'p')
  vim.fn.mkdir(W('deep/deeper/deepest/and/further'), 'p')
  vim.fn.mkdir(work .. '/home', 'p')

  local function body(tag)
    local t = {}
    for i = 1, 12 do
      t[i] = string.format('%s line %02d', tag, i)
    end
    return t
  end

  local files = {
    { 'alpha.txt', 'alpha' },
    { 'beta.txt', 'beta' },
    { 'gamma.txt', 'gamma' },
    { 'delta.txt', 'delta' },
    { 'epsilon.txt', 'epsilon' },
    { 'zeta.txt', 'zeta' },
    { 'unlisted.txt', 'unlisted' },
    { 'sub1/dup.txt', 'dup1' },
    { 'sub2/dup.txt', 'dup2' },
    { 'h\u{e9}llo-\u{3b4}.txt', 'multibyte' },
    { 'two words.txt', 'spaced' },
    { 'deep/deeper/deepest/and/further/long.txt', 'long' },
    { 'home/inhome.txt', 'inhome' },
    { 'abs.txt', 'abs' },
    { 'modeline.txt', 'modeline' },
    { 'twin.txt', 'twin' },
  }
  for _, f in ipairs(files) do
    writefile(f[1], body(f[2]))
  end

  local function add(name, listed, load)
    local b = vim.fn.bufadd(name)
    if load then
      vim.fn.bufload(b)
    end
    vim.api.nvim_set_option_value('buflisted', listed, { buf = b })
    return b
  end

  FIX.alpha = add('alpha.txt', true, true) -- 2
  FIX.beta = add('beta.txt', true, true) -- 3
  vim.api.nvim_buf_set_lines(FIX.beta, 0, -1, false, { 'beta CHANGED' })
  FIX.gamma = add('gamma.txt', true, true) -- 4
  vim.api.nvim_exec2('silent! ' .. FIX.gamma .. 'bunload', { output = false })
  FIX.delta = add('delta.txt', true, false) -- 5
  FIX.epsilon = add('epsilon.txt', true, true) -- 6
  vim.api.nvim_set_option_value('readonly', true, { buf = FIX.epsilon })
  FIX.zeta = add('zeta.txt', true, true) -- 7
  vim.api.nvim_set_option_value('modifiable', false, { buf = FIX.zeta })
  FIX.unlisted = add('unlisted.txt', false, true) -- 8

  FIX.termrun = vim.api.nvim_create_buf(true, true) -- 9
  vim.api.nvim_buf_set_name(FIX.termrun, 'term-run')
  FIX.termchan = vim.api.nvim_open_term(FIX.termrun, {})
  FIX.termdone = vim.api.nvim_create_buf(true, true) -- 10
  vim.api.nvim_buf_set_name(FIX.termdone, 'term-done')
  local ch = vim.api.nvim_open_term(FIX.termdone, {})
  pcall(vim.fn.chanclose, ch)

  FIX.prompt = vim.api.nvim_create_buf(true, true) -- 11
  vim.api.nvim_set_option_value('buftype', 'prompt', { buf = FIX.prompt })
  FIX.scratch = vim.api.nvim_create_buf(true, true) -- 12

  FIX.dup1 = add('sub1/dup.txt', true, true) -- 13
  FIX.dup2 = add('sub2/dup.txt', true, true) -- 14
  FIX.multi = add('h\u{e9}llo-\u{3b4}.txt', true, true) -- 15
  FIX.spaced = add('two words.txt', true, true) -- 16
  FIX.long = add('deep/deeper/deepest/and/further/long.txt', true, true) -- 17
  FIX.inhome = add(work .. '/home/inhome.txt', true, true) -- 18
  FIX.abs = add(work .. '/abs.txt', true, true) -- 19
  -- b7 re-`:edit!`s this one per case; it is added (not edited) here so
  -- that buffer 1 stays the startup `[No Name]`.  `curbuf_reusable()`
  -- would otherwise hand `:edit` the empty unnamed buffer and there
  -- would be no `[No Name]` left for b6 to ask `CTRL-G` about.
  FIX.modeline = add('modeline.txt', true, true) -- 20
  FIX.twin = add('twin.txt', true, true) -- 21

  -- Four ENTERING steps, each after a tick: four strictly ordered
  -- `b_last_used` values and no tie anywhere.  See `tick()`.
  tick()
  vim.fn.setqflist({ { filename = W('alpha.txt'), lnum = 2, text = 'qf one' } })
  vim.api.nvim_exec2('silent! copen', { output = false })
  FIX.qf = vim.fn.bufnr('%') -- 20
  vim.api.nvim_exec2('silent! wincmd p', { output = false })
  vim.api.nvim_exec2('silent! cclose', { output = false })

  tick()
  vim.api.nvim_exec2('silent! help help', { output = false })
  FIX.help = vim.fn.bufnr('%') -- 23
  vim.api.nvim_exec2('silent! helpclose', { output = false })

  tick()
  vim.api.nvim_set_current_buf(FIX.modeline)

  tick()
  vim.api.nvim_set_current_buf(FIX.alpha)
  vim.api.nvim_exec2('silent! balt ' .. W('beta.txt'), { output = false })
  baseopts()
end

fixture()

-- ========================================================== b0 defaults

section('b0-defaults', function()
  local opts = {
    'modeline',
    'modelines',
    'modelineexpr',
    'hidden',
    'bufhidden',
    'buflisted',
    'buftype',
    'switchbuf',
    'autowrite',
    'autoread',
    'wildignorecase',
    'fileignorecase',
    'report',
    'shortmess',
    'ruler',
    'readonly',
    'modifiable',
    'modified',
    'swapfile',
    'undofile',
    'previewheight',
    'helpheight',
  }
  for _, name in ipairs(opts) do
    local ok, v = pcall(function()
      return vim.api.nvim_get_option_value(name, {})
    end)
    ans('b0/opt/' .. name, ok and 'ok' or 'ERR', tostring(ok and v or v))
  end

  -- The minimum Vim version `chk_modeline` compares against.  b7's
  -- whole version-gate block is arithmetic on this one number.
  ans('b0/vimversion', 'v:version', tostring(vim.v.version), {
    version = vim.v.version,
    patch801 = vim.fn.has('patch-8.1.0000'),
  })

  -- The fixture roster: one row per buffer, so the twenty-three
  -- numbers below are pinned by name, not by position.
  for _, b in ipairs(vim.api.nvim_list_bufs()) do
    local function o(name)
      local ok, v = pcall(vim.api.nvim_get_option_value, name, { buf = b })
      return ok and tostring(v) or 'ERR'
    end
    local info = vim.fn.getbufinfo(b)[1] or {}
    ans(
      string.format('b0/buf/%02d', b),
      vim.fn.bufname(b),
      string.format(
        'bt=%s listed=%s loaded=%d mod=%s ro=%s ma=%s lines=%s win=%d lastused=%s',
        o('buftype'),
        o('buflisted'),
        vim.fn.bufloaded(b),
        o('modified'),
        o('readonly'),
        o('modifiable'),
        tostring(info.linecount),
        #(info.windows or {}),
        (info.lastused or 0) > 0 and 'set' or '0'
      ),
      {
        nr = b,
        name = vim.fn.bufname(b),
        bt = o('buftype'),
        listed = o('buflisted'),
        loaded = vim.fn.bufloaded(b),
        mod = o('modified'),
        ro = o('readonly'),
        ma = o('modifiable'),
        linecount = info.linecount,
        nwin = #(info.windows or {}),
        lastused = (info.lastused or 0) > 0 and 1 or 0,
      }
    )
  end
  ans('b0/triples', 'all', triples(30))
  ans('b0/curalt', 'start', curalt())
end)

-- =========================================================== b1 lsflags

section('b1-lsflags', function()
  local FLAGS = { '!', '+', '-', '=', 'a', 'u', 'h', 'x', '%', '#', 'R', 'F', 't' }
  for _, f in ipairs(FLAGS) do
    ans('b1/one/' .. f, 'ls ' .. f, lsout('ls ' .. f))
    ans('b1/one/bang/' .. f, 'ls! ' .. f, lsout('ls! ' .. f))
  end
  -- Every pair of the FILTER flags (the sort flag `t` is handled
  -- below): `buflist_list`'s giant disjunction is the only place the
  -- combinations interact, and a pair is what separates "or" from
  -- "and".
  local PAIRS = { '+', '-', '=', 'a', 'u', 'h', 'x', '%', '#', 'R', 'F' }
  for i = 1, #PAIRS do
    for j = i + 1, #PAIRS do
      local two = PAIRS[i] .. PAIRS[j]
      ans('b1/pair/' .. two, 'ls! ' .. two, lsout('ls! ' .. two))
    end
  end
  -- The sort flag against each filter.
  for _, f in ipairs(PAIRS) do
    ans('b1/sort/' .. f, 'ls! t' .. f, lsout('ls! t' .. f))
  end
  ans('b1/plain/ls', 'ls', lsout('ls'))
  ans('b1/plain/lsbang', 'ls!', lsout('ls!'))
  ans('b1/plain/files', 'files', lsout('files'))
  ans('b1/plain/filesbang', 'files!', lsout('files!'))
  ans('b1/plain/buffers', 'buffers', lsout('buffers'))
  ans('b1/plain/buffersbang', 'buffers!', lsout('buffers!'))
  ans('b1/plain/t', 'ls t', lsout('ls t'))
  ans('b1/plain/bang-t', 'ls! t', lsout('ls! t'))
  -- Unknown flags are not an error: `buflist_list` only ever asks
  -- `vim_strchr` whether a given letter is present.
  ans('b1/odd/zqv', 'ls! zqv', lsout('ls! zqv'))
  ans('b1/odd/dup', 'ls! uuuu', lsout('ls! uuuu'))
  ans('b1/odd/space', 'ls!  u ', lsout('ls!  u '))
  ans('b1/odd/all', 'ls! +-=auhx%#RFt', lsout('ls! +-=auhx%#RFt'))
  -- `message_filtered` is consulted per line, on NameBuff.
  ans('b1/filter/dup', 'filter /dup/ ls!', lsout('filter /dup/ ls!'))
  ans('b1/filter/inv', 'filter! /dup/ ls!', lsout('filter! /dup/ ls!'))
  ans('b1/filter/none', 'filter /nomatch/ ls!', lsout('filter /nomatch/ ls!'))
  ans('b1/filter/term', 'filter /term/ ls!', lsout('filter /term/ ls!'))
  -- The 40-column pad is `40 - vim_strsize(IObuff)`, and `vim_strsize`
  -- counts CELLS: the multibyte name is the only row that separates a
  -- byte count from a cell count.
  ans('b1/pad/multi', 'filter /hello/ ls!', lsout('filter /h\u{e9}llo/ ls!'))
  ans('b1/pad/long', 'filter /long/ ls!', lsout('filter /long/ ls!'))
end)

-- =========================================================== b2 address

section('b2-address', function()
  local function home()
    vim.api.nvim_set_current_buf(FIX.alpha)
    vim.api.nvim_exec2('silent! balt ' .. W('beta.txt'), { output = false })
  end

  -- `:buffer N` -- every fixture number, plus out of range and 0.
  for _, n in ipairs({ 1, 2, 5, 8, 9, 11, 12, 20, 21, 23, 0, -1, 99, 2147483647 }) do
    home()
    run('b2/bufnr/' .. tostring(n), 'buffer ' .. n)
  end
  -- `:buffer` with no argument, `#`, `%`, and a count as a range.
  home()
  run('b2/bare', 'buffer')
  home()
  run('b2/hash', 'buffer #')
  home()
  run('b2/percent', 'buffer %')
  home()
  run('b2/range', '5buffer')
  home()
  run('b2/range-bang', '5buffer!')

  -- `:buffer <name-fragment>` -- `buflist_findpat` from the ex command.
  for _, pat in ipairs({
    'gamma',
    'gam',
    'dup',
    'sub1/dup',
    '^gamma',
    'gamma$',
    '^gamma.txt$',
    'unlisted',
    'nosuch',
    'two words',
    'term',
    'term-run',
  }) do
    home()
    run('b2/name/' .. pat:gsub('[^%w]', '_'), 'buffer ' .. pat)
  end

  -- The walk: `:bnext`/`:bprev`/`:bfirst`/`:blast` with counts, and the
  -- wrap at both ends.  `do_buffer_ext`'s count loop is the only place
  -- an unlisted buffer is skipped and the only place the wrap happens.
  for _, spec in ipairs({
    { 'bnext', '' },
    { 'bnext', '2' },
    { 'bnext', '5' },
    { 'bnext', '30' },
    { 'bprevious', '' },
    { 'bprevious', '3' },
    { 'bprevious', '30' },
    { 'bNext', '' },
    { 'bfirst', '' },
    { 'blast', '' },
    { 'brewind', '' },
  }) do
    home()
    run(
      'b2/walk/' .. spec[1] .. (spec[2] == '' and '' or ('-' .. spec[2])),
      spec[2] .. spec[1]
    )
  end
  -- The same walk started from the LAST listed buffer, so the wrap is
  -- the answer rather than the step.
  vim.api.nvim_exec2('silent! blast', { output = false })
  run('b2/wrap/next', 'bnext')
  vim.api.nvim_exec2('silent! bfirst', { output = false })
  run('b2/wrap/prev', 'bprevious')

  home()
  run('b2/bmod', 'bmodified')
  home()
  run('b2/bmod2', '2bmodified')
  home()
  run('b2/bmod-back', { 'bmodified', 'bmodified' })

  -- CTRL-^ is `:buffer #` through a different door (`nv_hat`), and the
  -- count form is `:buffer N`.
  home()
  run('b2/hat/exe', 'exe "normal! \\<C-^>"')
  home()
  run('b2/hat/count', 'exe "normal! 5\\<C-^>"')
  home()
  run('b2/hat/zero', 'exe "normal! 0\\<C-^>"')

  -- `:balt` sets the alternate without entering; `:badd +lnum` sets the
  -- remembered cursor line (`buflist_setfpos`).
  home()
  run('b2/balt/name', { 'balt ' .. W('zeta.txt'), 'buffer #' })
  home()
  run('b2/balt/nr', { 'balt ' .. W('gamma.txt'), 'buffer #' })
  home()
  run(
    'b2/badd/lnum',
    { 'badd +7 ' .. W('gamma.txt'), 'buffer gamma' },
    'line=' .. vim.fn.line('.')
  )
  home()
  run(
    'b2/badd/again',
    { 'badd +3 ' .. W('alpha.txt') },
    'line=' .. vim.fn.line('.')
  )

  -- `:sbuffer` splits; the window count is part of the answer.
  local function wins()
    return 'wins=' .. vim.fn.winnr('$')
  end
  home()
  run('b2/sbuffer/nr', { 'sbuffer 5', 'only' }, wins)
  home()
  run('b2/sbuffer/name', { 'sbuffer gamma', 'only' }, wins)
  home()
  run('b2/sbuffer/bad', { 'sbuffer nosuch', 'only' }, wins)
  home()
  run('b2/sbnext', { 'sbnext', 'only' }, wins)

  -- `:bufdo` walks the listed buffers only, and leaves you on the last.
  home()
  run('b2/bufdo/count', 'bufdo echo bufnr("%")')
  home()
  run('b2/bufdo/err', 'bufdo call nosuchfunc()')

  -- `:[range]buffer` forms that address a RANGE of the buffer list.
  home()
  run('b2/argu/unknown', 'buffer +bad gamma')
  home()
  run('b2/switchbuf/useopen', {
    'set switchbuf=useopen',
    'sbuffer 5',
    'sbuffer 5',
    'only',
    'set switchbuf=',
  }, wins)
  home()
  baseopts()
end)

-- ========================================================= b3 lifecycle
-- ONE CHILD PER CASE.  Every child starts at bufnr 1, so the numbers
-- below are stable and an inserted case renumbers nothing.

section('b3-lifecycle', function()
  local CASES = {
    { 'bdelete-nr', "FIX(); OUT(DO({'bdelete 3'}), LS(''), LS('!'), TRIP(), CUR())" },
    { 'bdelete-cur', "FIX(); vim.cmd('buffer 2'); OUT(DO({'bdelete'}), LS('!'), TRIP(), CUR())" },
    { 'bdelete-name', "FIX(); OUT(DO({'bdelete two.txt'}), LS('!'), TRIP(), CUR())" },
    { 'bdelete-range', "FIX(); OUT(DO({'2,4bdelete'}), LS('!'), TRIP(), CUR())" },
    { 'bdelete-list', "FIX(); OUT(DO({'bdelete 3 5 7'}), LS('!'), TRIP(), CUR())" },
    { 'bdelete-mod', "FIX(); OUT(DO({'bdelete 4'}), LS('!'), TRIP(), CUR())" },
    { 'bdelete-mod-bang', "FIX(); OUT(DO({'bdelete! 4'}), LS('!'), TRIP(), CUR())" },
    { 'bdelete-unloaded', "FIX(); OUT(DO({'bdelete 5'}), LS('!'), TRIP(), CUR())" },
    { 'bdelete-unlisted', "FIX(); OUT(DO({'bdelete 6'}), LS('!'), TRIP(), CUR())" },
    { 'bdelete-twice', "FIX(); OUT(DO({'bdelete 3', 'bdelete 3'}), LS('!'), TRIP(), CUR())" },
    { 'bdelete-all', "FIX(); OUT(DO({'1,7bdelete'}), LS('!'), TRIP(), CUR())" },
    { 'bwipe-nr', "FIX(); OUT(DO({'bwipeout 3'}), LS('!'), TRIP(), CUR())" },
    { 'bwipe-cur', "FIX(); vim.cmd('buffer 2'); OUT(DO({'bwipeout'}), LS('!'), TRIP(), CUR())" },
    { 'bwipe-mod', "FIX(); OUT(DO({'bwipeout 4'}), LS('!'), TRIP(), CUR())" },
    { 'bwipe-mod-bang', "FIX(); OUT(DO({'bwipeout! 4'}), LS('!'), TRIP(), CUR())" },
    { 'bwipe-range', "FIX(); OUT(DO({'2,4bwipeout'}), LS('!'), TRIP(), CUR())" },
    { 'bwipe-after-bdelete', "FIX(); OUT(DO({'bdelete 3', 'bwipeout 3'}), LS('!'), TRIP(), CUR())" },
    { 'bwipe-all', "FIX(); OUT(DO({'1,7bwipeout'}), LS('!'), TRIP(), CUR())" },
    { 'bunload-nr', "FIX(); OUT(DO({'bunload 3'}), LS('!'), TRIP(), CUR())" },
    { 'bunload-cur', "FIX(); vim.cmd('buffer 2'); OUT(DO({'bunload'}), LS('!'), TRIP(), CUR())" },
    { 'bunload-mod', "FIX(); OUT(DO({'bunload 4'}), LS('!'), TRIP(), CUR())" },
    { 'bunload-mod-bang', "FIX(); OUT(DO({'bunload! 4'}), LS('!'), TRIP(), CUR())" },
    { 'bunload-unloaded', "FIX(); OUT(DO({'bunload 5'}), LS('!'), TRIP(), CUR())" },
    { 'edit-after-bdelete', "FIX(); OUT(DO({'bdelete 3', 'edit two.txt'}), LS('!'), TRIP(), CUR())" },
    { 'edit-after-bwipe', "FIX(); OUT(DO({'bwipeout 3', 'edit two.txt'}), LS('!'), TRIP(), CUR())" },
    { 'bdelete-in-window', "FIX(); OUT(DO({'split', 'buffer 3', 'wincmd p', 'bdelete 3'}), LS('!'), TRIP(), CUR())" },
    { 'bwipe-in-two-windows', "FIX(); OUT(DO({'split', 'buffer 3', 'wincmd p', 'buffer 3', 'bwipeout 3'}), LS('!'), TRIP(), CUR())" },
    { 'bdelete-last-listed', "FIX(); OUT(DO({'1,6bdelete', 'bdelete 7'}), LS('!'), TRIP(), CUR())" },
    { 'jumplist', "FIX(); OUT(DO({'buffer 3', 'buffer 4', 'bdelete 3'}), LS('!'), TRIP(), CUR(), vim.inspect(#vim.fn.getjumplist()[1]))" },
    { 'alt-after-bwipe', "FIX(); OUT(DO({'buffer 3', 'buffer 2', 'bwipeout 3'}), LS('!'), TRIP(), CUR())" },
    { 'bufnr-create', "FIX(); local n = vim.fn.bufnr('brandnew.txt', 1); OUT('created=' .. n, LS('!'), TRIP(), CUR())" },
    { 'badd-then-wipe', "FIX(); OUT(DO({'badd +2 six.txt', 'bwipeout six.txt'}), LS('!'), TRIP(), CUR())" },
    { 'setbuflisted', "FIX(); OUT(DO({'set nobuflisted', 'ls!'}), LS(''), TRIP(), CUR())" },
    { 'bufhidden-wipe', "FIX(); OUT(DO({'buffer 3', 'setlocal bufhidden=wipe', 'buffer 2'}), LS('!'), TRIP(), CUR())" },
    { 'bufhidden-delete', "FIX(); OUT(DO({'buffer 3', 'setlocal bufhidden=delete', 'buffer 2'}), LS('!'), TRIP(), CUR())" },
    { 'bufhidden-unload', "FIX(); OUT(DO({'buffer 3', 'setlocal bufhidden=unload', 'buffer 2'}), LS('!'), TRIP(), CUR())" },
    { 'nohidden-bnext', "FIX(); OUT(DO({'set nohidden', 'buffer 4', 'bnext'}), LS('!'), TRIP(), CUR())" },
    { 'file-rename', "FIX(); OUT(DO({'buffer 3', 'file renamed.txt'}), LS('!'), TRIP(), CUR())" },
    { 'file-rename-existing', "FIX(); OUT(DO({'buffer 3', 'file one.txt'}), LS('!'), TRIP(), CUR())" },
    { 'saveas', "FIX(); OUT(DO({'buffer 3', 'saveas! saved.txt'}), LS('!'), TRIP(), CUR())" },
    -- `:badd` on a buffer that already exists, is INITIALISED and is
    -- UNLISTED is the only gesture that reaches `buflist_new`'s
    -- `BLN_LISTED` re-listing block.  `:edit` on a `:bdelete`d buffer
    -- looks like it should, and does not: `close_buffer` clears
    -- `b_p_initialized`, so the `buf_copy_options()` call ten lines
    -- ABOVE that block re-initialises `'buflisted'` from the global
    -- first and the block is skipped.  Buffer 6 is `nvim_create_buf`'s
    -- unlisted scratch, which is initialised, so it survives the copy
    -- still unlisted.
    { 'badd-unlisted', "FIX(); OUT(DO({'badd scratch-unlisted'}), LS(''), LS('!'), TRIP(), CUR())" },
  }
  for _, c in ipairs(CASES) do
    case('b3/' .. c[1], c[2])
  end
end)

-- =========================================================== b4 findpat

section('b4-findpat', function()
  local PATS = {
    'alpha',
    'alph',
    'ALPHA',
    'a',
    'dup',
    'sub1',
    'sub1/dup',
    '^alpha',
    'alpha$',
    '^alpha.txt$',
    '^dup',
    'dup.txt$',
    '.txt$',
    'txt',
    'nosuchbuffer',
    'unlisted',
    'unlisted.txt',
    'term',
    'term-run',
    'term-done',
    '%',
    '#',
    'two words',
    'h\u{e9}llo',
    'long',
    'further/long',
    'inhome',
    'abs',
    'helphelp',
    'Quickfix',
    'Prompt',
    'modeline',
    '\\.txt',
    '*',
    '?',
    '[',
  }
  for _, p in ipairs(PATS) do
    local key = p:gsub('[^%w]', function(c)
      return string.format('%%%02x', c:byte())
    end)
    local okn, nr = pcall(vim.fn.bufnr, p)
    local okm, nm = pcall(vim.fn.bufname, p)
    -- The E93/E94 text is emitted by `buflist_findpat` itself, so it is
    -- part of the answer, not an exception.
    local msgs = vim.api.nvim_exec2('echo bufnr(' .. vim.fn.string(p) .. ')', {
      output = true,
    })
    ans(
      'b4/pat/' .. key,
      p,
      string.format(
        'bufnr=%s bufname=%s echo=%s',
        okn and tostring(nr) or errtext(nr),
        okm and tostring(nm) or errtext(nm),
        (msgs.output or ''):gsub('^\n', ''):gsub('\r?\n', ' | ')
      )
    )
  end
  -- `unlisted` is the second argument of `buflist_findpat` and the
  -- fallback loop is the only thing that reads it.
  for _, p in ipairs({ 'unlisted', 'helphelp', 'alpha' }) do
    ans(
      'b4/unlisted/' .. p,
      p,
      string.format(
        'bufnr=%s bufwinnr=%s bufexists=%s buflisted=%s bufloaded=%s',
        vim.fn.bufnr(p),
        vim.fn.bufwinnr(p),
        vim.fn.bufexists(p),
        vim.fn.buflisted(p),
        vim.fn.bufloaded(p)
      )
    )
  end
  -- `'fileignorecase'` and `'wildignorecase'` change `fname_match`'s
  -- `rm_ic`, which is a different door into the same regex.
  for _, spec in ipairs({
    { 'fic', 'set fileignorecase', 'set nofileignorecase' },
    { 'wic', 'set wildignorecase', 'set nowildignorecase' },
    { 'both', 'set fileignorecase wildignorecase', 'set nofileignorecase nowildignorecase' },
  }) do
    vim.api.nvim_exec2(spec[2], { output = false })
    ans(
      'b4/case/' .. spec[1],
      spec[2],
      string.format(
        'ALPHA=%d Alpha=%d alpha=%d DUP=%d',
        vim.fn.bufnr('ALPHA'),
        vim.fn.bufnr('Alpha'),
        vim.fn.bufnr('alpha'),
        vim.fn.bufnr('DUP')
      )
    )
    vim.api.nvim_exec2(spec[3], { output = false })
  end
  -- `bufname()`/`bufnr()` on a NUMBER take `buflist_findnr`, not the
  -- pattern path; 0 means the alternate.
  for _, n in ipairs({ 0, 1, 2, 5, 8, 20, 21, 99, -1 }) do
    ans(
      'b4/nr/' .. tostring(n),
      tostring(n),
      string.format(
        'bufname=%s bufexists=%d buflisted=%d bufloaded=%d bufwinid=%d',
        vim.fn.bufname(n),
        vim.fn.bufexists(n),
        vim.fn.buflisted(n),
        vim.fn.bufloaded(n),
        vim.fn.bufwinid(n)
      )
    )
  end
end)

-- ========================================================== b5 complete

section('b5-complete', function()
  local PATS = {
    '',
    'a',
    'al',
    'alpha',
    'd',
    'dup',
    'sub',
    '^a',
    '^dup',
    'txt',
    '.txt',
    'z',
    'nosuch',
    't',
    'term',
    'two',
    'h\u{e9}',
    'deep',
    'long',
    '*',
    '.',
  }
  for _, p in ipairs(PATS) do
    local key = p == '' and 'empty' or p:gsub('[^%w]', function(c)
      return string.format('%%%02x', c:byte())
    end)
    local ok, res = pcall(vim.fn.getcompletion, p, 'buffer')
    ans(
      'b5/pat/' .. key,
      p,
      ok and table.concat(res, ' ') or errtext(res),
      { pat = p, ok = ok, matches = ok and res or tostring(res) }
    )
  end
  -- `ExpandBufnames`'s LAST block rotates the current buffer to the END
  -- of the list when `WILD_BUFLASTUSED` is on -- which is what
  -- `:buffer <Tab>` uses and `getcompletion()` does not.  The command
  -- line is the only door to it.
  for _, cmd in ipairs({
    'buffer a',
    'buffer ',
    'buffer d',
    'sbuffer a',
    'bdelete a',
    'buffer t',
  }) do
    local ok, res = pcall(vim.fn.getcompletion, cmd, 'cmdline')
    ans(
      'b5/cmdline/' .. cmd:gsub('[^%w]', '_'),
      cmd,
      ok and table.concat(res, ' ') or errtext(res)
    )
  end
  for _, spec in ipairs({
    { 'wic', 'set wildignorecase', 'set nowildignorecase' },
    { 'fic', 'set fileignorecase', 'set nofileignorecase' },
  }) do
    vim.api.nvim_exec2(spec[2], { output = false })
    for _, p in ipairs({ 'A', 'AL', 'DUP', 'Sub' }) do
      ans(
        'b5/' .. spec[1] .. '/' .. p,
        spec[2] .. ' ' .. p,
        table.concat(vim.fn.getcompletion(p, 'buffer'), ' ')
      )
    end
    vim.api.nvim_exec2(spec[3], { output = false })
  end
  ans(
    'b5/diff/off',
    'getcompletion in a non-diff window',
    table.concat(vim.fn.getcompletion('a', 'diff_buffer'), ' ')
  )

  -- `ExpandBufnames`'s LAST BLOCK -- the `qsort` on `b_last_used` and
  -- the rotation that puts the CURRENT buffer at the END -- runs only
  -- under `WILD_BUFLASTUSED`, which `command_line_wildchar_complete`
  -- sets only when the active `'wildmode'` stage carries `lastused`.
  -- `getcompletion()` never passes it, so nothing above reaches those
  -- thirty lines: it takes a real `<Tab>` at a real command line.
  -- That redraws the cmdline, and in `--headless` the redraw goes to
  -- STDOUT with no newline anywhere, which is why these run in
  -- children whose answer comes back through a file.
  -- The answer is which buffer the Nth `<Tab>` picks: the completion is
  -- ACCEPTED with `<CR>`, so `bufname('%')` afterwards names the Nth
  -- match and the whole ordering falls out of six rows.
  local WM = [==[
FIX(); vim.cmd('buffer %d')
vim.o.wildmenu = true
vim.o.wildmode = '%s'
local ok, e = pcall(vim.fn.feedkeys, ':buffer %s' .. string.rep('\t', %d) .. '\r', 'ntx')
OUT('wm=%s', 'tabs=%d', 'ok=' .. tostring(ok) .. (ok and '' or (' ' .. tostring(e):gsub('\r?\n', ' '))),
  'cur=' .. vim.fn.bufnr('%%'), 'name=' .. vim.fn.bufname('%%'))
]==]
  for _, wm in ipairs({ 'full', 'full:lastused', 'longest:full,full:lastused' }) do
    for _, tabs in ipairs({ 1, 2, 3, 4, 5, 6 }) do
      case(
        'b5/wm/' .. wm:gsub('[^%w]', '_') .. '/' .. tabs,
        string.format(WM, 3, wm, 't', tabs, wm, tabs),
        true
      )
    end
  end
  -- ... and the same with the current buffer OUTSIDE the match set, so
  -- the rotation has nothing to rotate.
  for _, tabs in ipairs({ 1, 2, 3 }) do
    case(
      'b5/wm/outside/' .. tabs,
      string.format(WM, 6, 'full:lastused', 'o', tabs, 'full:lastused', tabs),
      true
    )
  end
end)

-- ========================================================== b6 fileinfo

section('b6-fileinfo', function()
  local BUFS = {
    { 'noname', 1 },
    { 'alpha', nil },
    { 'beta-mod', nil },
    { 'epsilon-ro', nil },
    { 'zeta-noma', nil },
    { 'help', nil },
    { 'term', nil },
    { 'quickfix', nil },
    { 'scratch', nil },
    { 'multi', nil },
    { 'inhome', nil },
    { 'abs', nil },
    { 'long', nil },
    { 'spaced', nil },
  }
  BUFS[2][2] = FIX.alpha
  BUFS[3][2] = FIX.beta
  BUFS[4][2] = FIX.epsilon
  BUFS[5][2] = FIX.zeta
  BUFS[6][2] = FIX.help
  BUFS[7][2] = FIX.termdone
  BUFS[8][2] = FIX.qf
  BUFS[9][2] = FIX.scratch
  BUFS[10][2] = FIX.multi
  BUFS[11][2] = FIX.inhome
  BUFS[12][2] = FIX.abs
  BUFS[13][2] = FIX.long
  BUFS[14][2] = FIX.spaced

  local GESTURES = {
    { 'file', 'file' },
    { 'ctrlg', 'exe "normal! \\<C-g>"' },
    { 'ctrlg1', 'exe "normal! 1\\<C-g>"' },
    { 'ctrlg2', 'exe "normal! 2\\<C-g>"' },
    { 'gctrlg', 'exe "normal! g\\<C-g>"' },
  }
  for _, b in ipairs(BUFS) do
    vim.api.nvim_set_current_buf(b[2])
    pcall(vim.api.nvim_win_set_cursor, 0, { math.min(3, vim.fn.line('$')), 0 })
    for _, g in ipairs(GESTURES) do
      local ok, res = pcall(vim.api.nvim_exec2, g[2], { output = true })
      ans(
        'b6/' .. b[1] .. '/' .. g[1],
        g[2],
        ok and ((res.output or ''):gsub('^\n', ''):gsub('\r?\n', ' | '))
          or errtext(res)
      )
    end
  end

  -- `'shortmess'` changes which of the six `%s` slots `fileinfo`
  -- fills, and `'ruler'` swaps the whole tail for a percentage.
  for _, sm in ipairs({
    'aoOtTIcCF',
    '',
    'm',
    'r',
    'mr',
    'filnxtToOF',
    'aoOtTIcCFs',
  }) do
    vim.api.nvim_exec2('set shortmess=' .. sm, { output = false })
    -- `SHM_MOD` swaps `[Modified]` for `[+]` and `SHM_RO`
    -- `[readonly]` for `[RO]`; one buffer cannot show both.
    for _, b in ipairs({ { 'mod', FIX.beta }, { 'ro', FIX.epsilon } }) do
      vim.api.nvim_set_current_buf(b[2])
      for _, g in ipairs({ 'file', 'exe "normal! \\<C-g>"' }) do
        local ok, res = pcall(vim.api.nvim_exec2, g, { output = true })
        ans(
          'b6/shm/'
            .. (sm == '' and 'empty' or sm)
            .. '/'
            .. b[1]
            .. '/'
            .. (g == 'file' and 'file' or 'ctrlg'),
          'shortmess=' .. sm,
          ok and ((res.output or ''):gsub('^\n', ''):gsub('\r?\n', ' | '))
            or errtext(res)
        )
      end
    end
  end
  vim.api.nvim_exec2('set shortmess=aoOtTIcCF', { output = false })
  for _, ru in ipairs({ 'set ruler', 'set noruler' }) do
    vim.api.nvim_exec2(ru, { output = false })
    for _, b in ipairs({ FIX.alpha, FIX.scratch }) do
      vim.api.nvim_set_current_buf(b)
      local ok, res = pcall(vim.api.nvim_exec2, 'file', { output = true })
      ans(
        'b6/ruler/' .. (ru:match('noruler') and 'off' or 'on') .. '/' .. b,
        ru,
        ok and ((res.output or ''):gsub('^\n', ''):gsub('\r?\n', ' | '))
          or errtext(res)
      )
    end
  end
  baseopts()
  vim.api.nvim_set_current_buf(FIX.alpha)
end)

-- ========================================================== b7 modeline

section('b7-modeline', function()
  local FILE = W('modeline.txt')
  local BODY = { 'body one', 'body two', 'body three', 'body four', 'body five' }

  --- The options a modeline may reach.  `sw`/`ts`/`tw` are plain
  --- numbers, `ai` a boolean, `com` a string, `fdm` a per-window one
  --- and `fde` an EXPRESSION option -- the last is the whole point of
  --- `'modelineexpr'`.
  local function opts()
    local function g(name, scope)
      local ok, v = pcall(vim.api.nvim_get_option_value, name, scope or {})
      return ok and tostring(v) or 'ERR'
    end
    return string.format(
      'sw=%s ts=%s tw=%s ai=%s et=%s com=%s fdm=%s fde=%s ff=%s co=%s ml=%s mls=%s mle=%s',
      g('shiftwidth'),
      g('tabstop'),
      g('textwidth'),
      g('autoindent'),
      g('expandtab'),
      g('comments'),
      g('foldmethod'),
      g('foldexpr'),
      g('fileformat'),
      tostring(vim.o.columns),
      tostring(vim.o.modeline),
      tostring(vim.o.modelines),
      tostring(vim.o.modelineexpr)
    )
  end

  -- Every option a modeline may have set is buffer- or window-LOCAL
  -- and the sweep re-uses ONE buffer, so without a reset each case
  -- inherits the previous one's answer (`'comments'` stayed at `b:#`
  -- for thirty rows the first time), and `-u NONE` means no ftplugin
  -- resets them either.  The reset SNAPSHOTS the values rather than
  -- writing `setlocal xxx&`, because `&` restores the COMPILED
  -- default, not what `BASEOPT` pinned -- and Nvim's default for
  -- `'autoindent'` is ON, so `setlocal autoindent&` after
  -- `set noautoindent` turns it back on and every row reads `ai=true`.
  local RESETOPTS = {
    'shiftwidth',
    'tabstop',
    'textwidth',
    'autoindent',
    'expandtab',
    'comments',
    'foldmethod',
    'foldexpr',
    'fileformat',
    'commentstring',
    'formatoptions',
    'indentexpr',
    'foldtext',
  }
  -- Enter the buffer BEFORE anything writes the file: entering a
  -- buffer whose file changed under it is W11, a warning that goes to
  -- stderr and would appear exactly once, from the first case.
  vim.api.nvim_set_current_buf(FIX.modeline)
  local PRISTINE = {}
  for _, name in ipairs(RESETOPTS) do
    local ok, v = pcall(vim.api.nvim_get_option_value, name, {})
    if ok then
      PRISTINE[name] = v
    end
  end

  local function mcase(label, lines, pre)
    vim.fn.writefile(lines, FILE)
    -- `'modeline'` is BUFFER-local: `set modeline` while some other
    -- buffer is current writes the global and that buffer's copy, and
    -- the `:edit!` below then lands in a buffer whose own `b_p_ml` is
    -- still off.  Enter the target FIRST -- otherwise exactly the
    -- first case of the section silently measures nothing.
    vim.api.nvim_set_current_buf(FIX.modeline)
    baseopts()
    for name, v in pairs(PRISTINE) do
      pcall(vim.api.nvim_set_option_value, name, v, {})
    end
    -- `:edit!` on a file that changed on disk is W11, which is a
    -- WARNING the sweep would otherwise print once, on stderr, from
    -- the first case only.  `'autoread'` makes the reread silent; the
    -- modeline still runs, which is the whole point.
    vim.api.nvim_exec2('set modeline modelines=5 autoread', { output = false })
    for _, c in ipairs(pre or {}) do
      pcall(vim.api.nvim_exec2, c, { output = false })
    end
    -- The buffer already exists (fixture 22), so `:edit!` re-reads it
    -- into the SAME buffer and creates nothing.
    local ok, res = pcall(vim.api.nvim_exec2, 'edit! ' .. FILE, { output = true })
    local note = ok and ((res.output or ''):gsub('^\n', ''):gsub('\r?\n', ' | '))
      or errtext(res)
    ans('b7/' .. label, note, opts(), {
      lines = lines,
      pre = pre or {},
      note = note,
      opts = opts(),
    })
  end

  local function first(text, extra)
    local l = { text }
    for _, b in ipairs(BODY) do
      l[#l + 1] = b
    end
    for _, e in ipairs(extra or {}) do
      l[#l + 1] = e
    end
    return l
  end
  local function last(text)
    local l = {}
    for _, b in ipairs(BODY) do
      l[#l + 1] = b
    end
    l[#l + 1] = text
    return l
  end

  -- The three spellings and the two forms.
  mcase('form/vim-set', first('/* vim: set sw=3 ts=3: */'))
  mcase('form/vim-se', first('/* vim: se sw=3: */'))
  mcase('form/vim-noset', first('/* vim: sw=3 ts=3 */'))
  mcase('form/vi-set', first('/* vi: set sw=3: */'))
  mcase('form/vi-noset', first('/* vi: sw=3 */'))
  mcase('form/ex-set', first('/* ex: set sw=3: */'))
  -- `ex:` at column ZERO is NOT a modeline: the C is
  -- `(prev != -1 && "ex:") || "vi:"`, so `ex:` needs a preceding
  -- character while `vi:` does not.
  mcase('form/ex-col0', first('ex: set sw=3:'))
  mcase('form/vi-col0', first('vi: set sw=3:'))
  mcase('form/vim-col0', first('vim: set sw=3:'))
  mcase('form/Vim-set', first('/* Vim: set sw=3: */'))
  -- ... and a capital `V` REQUIRES the word `set`.
  mcase('form/Vim-noset', first('/* Vim: sw=3 */'))
  mcase('form/VIM', first('/* VIM: set sw=3: */'))
  mcase('form/vim-nospace', first('/*vim: set sw=3: */'))
  mcase('form/vim-tab', first('\tvim: set sw=3:'))
  mcase('form/embedded', first('int x; /* vim: set sw=3: */'))
  mcase('form/novi', first('/* novi: set sw=3: */'))
  mcase('form/tvim', first('/* tvim: set sw=3: */'))

  -- The version gate.  `min_vim_version()` is 801.
  for _, v in ipairs({
    'vim<700:',
    'vim<801:',
    'vim<802:',
    'vim>700:',
    'vim>801:',
    'vim>802:',
    'vim=800:',
    'vim=801:',
    'vim=802:',
    'vim700:',
    'vim801:',
    'vim900:',
  }) do
    -- `vim<700:`, `vim>700:` and `vim=700:` differ in ONE character and
    -- a `[^%w] -> _` key collapses all three onto the same label.
    local key = v:gsub('<', 'lt'):gsub('>', 'gt'):gsub('=', 'eq'):gsub(':', '')
    mcase('vers/' .. key, first('/* ' .. v .. ' set sw=3: */'))
  end
  mcase('vers/noset', first('/* vim>=800: sw=3 */'))
  mcase('vers/Vim-vers', first('/* Vim801: set sw=3: */'))
  mcase('vers/Vim-vers-noset', first('/* Vim801: sw=3 */'))

  -- Placement: first N lines and last N lines, both ends of
  -- `'modelines'`.
  for _, n in ipairs({ 0, 1, 2, 5, 100 }) do
    mcase('mls/first/' .. n, first('/* vim: set sw=3: */'), {
      'set modelines=' .. n,
    })
    mcase('mls/last/' .. n, last('/* vim: set ts=3: */'), {
      'set modelines=' .. n,
    })
  end
  mcase('mls/line3', {
    'body one',
    'body two',
    '/* vim: set sw=3: */',
    'body four',
    'body five',
    'body six',
    'body seven',
    'body eight',
  }, { 'set modelines=2' })
  mcase('mls/line3-ok', {
    'body one',
    'body two',
    '/* vim: set sw=3: */',
    'body four',
    'body five',
    'body six',
    'body seven',
    'body eight',
  }, { 'set modelines=3' })
  mcase('mls/nomodeline', first('/* vim: set sw=3: */'), { 'set nomodeline' })
  mcase('mls/overlap', { '/* vim: set sw=3: */' }, { 'set modelines=5' })

  -- Malformed and hostile.
  mcase('bad/bare', first('/* vim: */'))
  mcase('bad/empty-set', first('/* vim: set: */'))
  mcase('bad/no-colon', first('/* vim: set sw=3 */'))
  mcase('bad/unknown-opt', first('/* vim: set nosuchoption=1: */'))
  mcase('bad/bad-value', first('/* vim: set sw=notanumber: */'))
  mcase('bad/escaped-colon', first('/* vim: set com=b\\:#: */'))
  mcase('bad/two-parts', first('/* vim: set sw=3:set ts=3: */'))
  mcase('bad/noset-two', first('/* vim: sw=3:ts=3 */'))
  mcase('bad/trailing', first('/* vim: set sw=3: trailing junk */'))
  mcase('bad/only-colon', first('/* vim::::: */'))
  mcase('bad/long', first('/* vim: set ' .. string.rep('sw=3 ', 300) .. ': */'))
  mcase('bad/nul-ish', first('/* vim: set sw=3\t: */'))
  mcase('bad/second-fails', first('/* vim: set sw=3:nosuchoption:ts=3: */'))

  -- The options a modeline is NOT allowed to set.  `secure` is 1 while
  -- `do_set` runs, so a P_SECURE option answers E520 and the whole
  -- modeline stops there.
  mcase('deny/secure', first('/* vim: set backupdir=/tmp: */'))
  mcase('deny/shell', first('/* vim: set shell=/bin/sh: */'))
  mcase('deny/global-only', first('/* vim: set columns=40: */'))
  mcase('deny/then-more', first('/* vim: set backupdir=/tmp sw=3: */'))

  -- `'modelineexpr'` off is E992 and reads exactly like a healthy row
  -- unless BOTH arms are explicit (b19-2's trap).
  mcase('mle/off-fde', first('/* vim: set foldexpr=1+1: */'))
  mcase('mle/on-fde', first('/* vim: set foldexpr=1+1: */'), {
    'set modelineexpr',
  })
  mcase('mle/off-inde', first('/* vim: set indentexpr=1: */'))
  mcase('mle/on-inde', first('/* vim: set indentexpr=1: */'), {
    'set modelineexpr',
  })
  mcase('mle/off-fdt', first('/* vim: set foldtext=getline(v:foldstart): */'))
  mcase('mle/on-fdt', first('/* vim: set foldtext=getline(v:foldstart): */'), {
    'set modelineexpr',
  })

  -- A window-local option through a modeline is `OPT_LOCAL`, so it
  -- lands on the window, not the global.
  mcase('scope/fdm', first('/* vim: set fdm=marker: */'))
  mcase('scope/ff', first('/* vim: set ff=mac: */'))
  mcase('scope/many', first('/* vim: set sw=3 ts=3 tw=44 ai et: */'))

  baseopts()
  vim.api.nvim_set_current_buf(FIX.alpha)
end)

-- =========================================================== b8 bufinfo

section('b8-bufinfo', function()
  --- `lastused` is a wall clock and `variables` carries every `b:` the
  --- runtime happened to set; both are normalised, everything else is
  --- the answer.
  local function norm(d)
    local out = {}
    for k, v in pairs(d) do
      if k == 'lastused' then
        out[k] = (v > 0) and 1 or 0
      elseif k == 'variables' then
        local keys = {}
        for vk in pairs(v) do
          keys[#keys + 1] = vk
        end
        table.sort(keys)
        out[k] = table.concat(keys, ',')
      elseif k == 'windows' then
        out[k] = #v
      elseif k == 'name' then
        out[k] = scrub(v)
      else
        out[k] = v
      end
    end
    return out
  end
  local function compact(d)
    local keys = {}
    for k in pairs(d) do
      keys[#keys + 1] = k
    end
    table.sort(keys)
    local parts = {}
    for _, k in ipairs(keys) do
      parts[#parts + 1] = k .. '=' .. tostring(d[k])
    end
    return table.concat(parts, ' ')
  end

  for _, b in ipairs(vim.api.nvim_list_bufs()) do
    local info = vim.fn.getbufinfo(b)[1]
    if info then
      local n = norm(info)
      ans(string.format('b8/info/%02d', b), vim.fn.bufname(b), compact(n), n)
    end
  end
  for _, sel in ipairs({
    { 'listed', { buflisted = 1 } },
    { 'loaded', { bufloaded = 1 } },
    { 'modified', { bufmodified = 1 } },
    { 'listed-loaded', { buflisted = 1, bufloaded = 1 } },
    -- An empty Lua table crosses as an empty LIST and `getbufinfo()`
    -- answers E745; `vim.empty_dict()` is the only way to ask for the
    -- unfiltered dict form.
    { 'all', vim.empty_dict() },
  }) do
    local list = vim.fn.getbufinfo(sel[2])
    local nrs = {}
    for i, d in ipairs(list) do
      nrs[i] = d.bufnr
    end
    ans('b8/sel/' .. sel[1], vim.inspect(sel[2]):gsub('%s+', ' '), table.concat(nrs, ' '))
  end
  ans('b8/one/nr', 'getbufinfo(5)', compact(norm(vim.fn.getbufinfo(5)[1] or {})))
  ans(
    'b8/one/name',
    'getbufinfo("gamma")',
    compact(norm(vim.fn.getbufinfo('gamma')[1] or {}))
  )
  ans('b8/one/bad', 'getbufinfo(999)', vim.inspect(vim.fn.getbufinfo(999)))

  -- `get/setbufvar` reach the same dict `getbufinfo().variables` shows,
  -- and the `&opt` form reaches the buffer's options.
  vim.fn.setbufvar(FIX.gamma, 'sweep', 'yes')
  vim.fn.setbufvar(FIX.gamma, '&tabstop', 3)
  ans(
    'b8/var/get',
    'gamma',
    string.format(
      'sweep=%s ts=%s missing=%s dflt=%s vars=%s',
      tostring(vim.fn.getbufvar(FIX.gamma, 'sweep')),
      tostring(vim.fn.getbufvar(FIX.gamma, '&tabstop')),
      vim.inspect(vim.fn.getbufvar(FIX.gamma, 'nosuch')),
      tostring(vim.fn.getbufvar(FIX.gamma, 'nosuch', 'dflt')),
      table.concat(
        (function()
          local keys = {}
          for k in pairs(vim.fn.getbufinfo(FIX.gamma)[1].variables) do
            keys[#keys + 1] = k
          end
          table.sort(keys)
          return keys
        end)(),
        ','
      )
    )
  )
  ans(
    'b8/var/bad',
    'setbufvar(999)',
    tostring(pcall(vim.fn.setbufvar, 999, 'x', 1))
  )
  vim.fn.setbufvar(FIX.gamma, '&tabstop', 8)

  ans('b8/triples', 'all', triples(30))
  ans(
    'b8/counts',
    'bufnr($)',
    string.format(
      'last=%d listed=%d loaded=%d total=%d',
      vim.fn.bufnr('$'),
      #vim.fn.getbufinfo({ buflisted = 1 }),
      #vim.fn.getbufinfo({ bufloaded = 1 }),
      #vim.api.nvim_list_bufs()
    )
  )
end)

-- =========================================================== b9 auorder
-- ONE CHILD PER GESTURE.  Buffer autocmds fire from a script, so no
-- main input loop is needed -- but the gestures churn the buffer list,
-- so each runs in its own editor and the numbers in its log start at 1.

section('b9-auorder', function()
  local HOOK = [[
_G.LOG = {}
local EVENTS = {'BufNew','BufAdd','BufCreate','BufDelete','BufWipeout',
  'BufUnload','BufEnter','BufLeave','BufHidden','BufWinEnter','BufWinLeave',
  'BufReadPre','BufReadPost','BufNewFile','BufFilePre','BufFilePost',
  'BufWritePre','BufWritePost','BufModifiedSet'}
for _, ev in ipairs(EVENTS) do
  vim.api.nvim_create_autocmd(ev, {callback = function(a)
    _G.LOG[#_G.LOG+1] = string.format('%s:%s', ev, tostring(a.buf))
  end})
end
function _G.SEQ() return table.concat(_G.LOG, ' > ') end
]]
  local CASES = {
    { 'badd', "FIX(); " .. HOOK .. " vim.cmd('badd six.txt'); OUT(SEQ(), TRIP(), CUR())" },
    { 'bufadd', "FIX(); " .. HOOK .. " vim.fn.bufadd('six.txt'); OUT(SEQ(), TRIP(), CUR())" },
    { 'createbuf', "FIX(); " .. HOOK .. " vim.api.nvim_create_buf(true, false); OUT(SEQ(), TRIP(), CUR())" },
    { 'createbuf-scratch', "FIX(); " .. HOOK .. " vim.api.nvim_create_buf(false, true); OUT(SEQ(), TRIP(), CUR())" },
    { 'edit-new', "FIX(); " .. HOOK .. " vim.cmd('edit six.txt'); OUT(SEQ(), TRIP(), CUR())" },
    { 'edit-existing', "FIX(); " .. HOOK .. " vim.cmd('edit one.txt'); OUT(SEQ(), TRIP(), CUR())" },
    { 'edit-missing', "FIX(); " .. HOOK .. " vim.cmd('edit nosuch.txt'); OUT(SEQ(), TRIP(), CUR())" },
    { 'enew', "FIX(); " .. HOOK .. " vim.cmd('enew'); OUT(SEQ(), TRIP(), CUR())" },
    { 'buffer-nr', "FIX(); " .. HOOK .. " vim.cmd('buffer 3'); OUT(SEQ(), TRIP(), CUR())" },
    { 'buffer-unloaded', "FIX(); " .. HOOK .. " vim.cmd('buffer 5'); OUT(SEQ(), TRIP(), CUR())" },
    { 'bnext', "FIX(); " .. HOOK .. " vim.cmd('bnext'); OUT(SEQ(), TRIP(), CUR())" },
    { 'bdelete', "FIX(); " .. HOOK .. " vim.cmd('bdelete 3'); OUT(SEQ(), TRIP(), CUR())" },
    { 'bdelete-cur', "FIX(); vim.cmd('buffer 2'); " .. HOOK .. " vim.cmd('bdelete'); OUT(SEQ(), TRIP(), CUR())" },
    { 'bwipeout', "FIX(); " .. HOOK .. " vim.cmd('bwipeout 3'); OUT(SEQ(), TRIP(), CUR())" },
    { 'bwipeout-cur', "FIX(); vim.cmd('buffer 2'); " .. HOOK .. " vim.cmd('bwipeout'); OUT(SEQ(), TRIP(), CUR())" },
    { 'bunload', "FIX(); vim.cmd('buffer 2'); " .. HOOK .. " vim.cmd('bunload'); OUT(SEQ(), TRIP(), CUR())" },
    { 'bufdelete-api', "FIX(); " .. HOOK .. " vim.api.nvim_buf_delete(3, {}); OUT(SEQ(), TRIP(), CUR())" },
    { 'bufdelete-force', "FIX(); " .. HOOK .. " vim.api.nvim_buf_delete(4, {force=true}); OUT(SEQ(), TRIP(), CUR())" },
    { 'split-buffer', "FIX(); " .. HOOK .. " vim.cmd('split'); vim.cmd('buffer 3'); OUT(SEQ(), TRIP(), CUR())" },
    { 'sbuffer', "FIX(); " .. HOOK .. " vim.cmd('sbuffer 3'); OUT(SEQ(), TRIP(), CUR())" },
    { 'file-rename', "FIX(); vim.cmd('buffer 2'); " .. HOOK .. " vim.cmd('file renamed.txt'); OUT(SEQ(), TRIP(), CUR())" },
    { 'bufdo', "FIX(); " .. HOOK .. " vim.cmd('bufdo echo 1'); OUT(SEQ(), TRIP(), CUR())" },
    { 'bufhidden-wipe', "FIX(); vim.cmd('buffer 3'); vim.cmd('setlocal bufhidden=wipe'); " .. HOOK .. " vim.cmd('buffer 2'); OUT(SEQ(), TRIP(), CUR())" },
    { 'modified-set', "FIX(); vim.cmd('buffer 2'); " .. HOOK .. " vim.api.nvim_buf_set_lines(0, 0, -1, false, {'x'}); OUT(SEQ(), TRIP(), CUR())" },
    { 'eventignore', "FIX(); " .. HOOK .. " vim.o.eventignore = 'BufEnter,BufLeave'; vim.cmd('buffer 3'); OUT(SEQ(), TRIP(), CUR())" },
    { 'nested-delete', "FIX(); " .. HOOK .. " vim.api.nvim_create_autocmd('BufDelete', {once=true, callback=function() pcall(vim.cmd, 'bdelete 5') end}); vim.cmd('bdelete 3'); OUT(SEQ(), TRIP(), CUR())" },
    { 'veto-bufleave', "FIX(); " .. HOOK .. " vim.api.nvim_create_autocmd('BufLeave', {once=true, callback=function() error('no') end}); pcall(vim.cmd, 'buffer 3'); OUT(SEQ(), TRIP(), CUR())" },
    -- BufAdd without BufNew: the buffer already exists, `:badd` only
    -- re-LISTS it.  See b3/badd-unlisted for why `:edit` on a deleted
    -- buffer does not reach the same block.
    { 'badd-unlisted', "FIX(); " .. HOOK .. " vim.cmd('badd scratch-unlisted'); OUT(SEQ(), TRIP(), CUR())" },
  }
  for _, c in ipairs(CASES) do
    case('b9/' .. c[1], c[2])
  end
end)

-- ======================================================== b91 crashprobe

section('b91-crashprobe', function()
  local CASES = {
    { 'bnext-huge', "FIX(); OUT(DO({'99999999bnext'}), CUR())" },
    { 'bprev-huge', "FIX(); OUT(DO({'99999999bprevious'}), CUR())" },
    { 'bnext-max', "FIX(); OUT(DO({'2147483647bnext'}), CUR())" },
    { 'buffer-max', "FIX(); OUT(DO({'2147483647buffer'}), CUR())" },
    { 'bdelete-range-max', "FIX(); OUT(DO({'1,2147483647bdelete'}), TRIP(), CUR())" },
    { 'bwipe-range-max', "FIX(); OUT(DO({'1,2147483647bwipeout'}), TRIP(), CUR())" },
    { 'bdelete-neg', "FIX(); OUT(DO({'bdelete -1'}), TRIP(), CUR())" },
    { 'bunload-range-max', "FIX(); OUT(DO({'1,2147483647bunload'}), TRIP(), CUR())" },
    { 'modelines-max', "FIX(); OUT(DO({'set modelines=2147483647', 'set modeline', 'edit! one.txt'}), CUR())" },
    { 'modelines-neg', "FIX(); OUT(DO({'set modelines=-1'}), CUR())" },
    { 'modeline-10k', "local s = 'vim: set ' .. string.rep('sw=3 ', 2000) .. ':'; vim.fn.writefile({s, 'body'}, 'huge.txt'); FIX(); OUT(DO({'set modeline modelines=5', 'edit! huge.txt'}), 'sw=' .. vim.o.shiftwidth)" },
    { 'modeline-deep-colon', "vim.fn.writefile({'vim:' .. string.rep('set sw=3:', 1000), 'body'}, 'colons.txt'); FIX(); OUT(DO({'set modeline modelines=5', 'edit! colons.txt'}), 'sw=' .. vim.o.shiftwidth)" },
    { 'longname', "FIX(); local n = string.rep('x', 4000) .. '.txt'; OUT(tostring(pcall(vim.cmd, 'badd ' .. n)), tostring(vim.fn.bufnr('$')))" },
    { 'many-buffers', "FIX(); for i = 1, 2000 do pcall(vim.fn.bufadd, 'g' .. i .. '.txt') end; OUT(tostring(vim.fn.bufnr('$')), tostring(#vim.api.nvim_list_bufs()))" },
    { 'many-then-ls', "FIX(); for i = 1, 400 do pcall(vim.fn.bufadd, 'g' .. i .. '.txt') end; local o = LS('!'); OUT(tostring(#o), tostring(vim.fn.bufnr('$')))" },
    { 'findpat-star', "FIX(); OUT(tostring(pcall(vim.fn.bufnr, string.rep('*', 200))), tostring(pcall(vim.fn.bufnr, '\\\\(')))" },
    { 'findpat-deep', "FIX(); OUT(tostring(pcall(vim.fn.bufnr, string.rep('a*', 500))))" },
    { 'complete-huge', "FIX(); for i = 1, 500 do pcall(vim.fn.bufadd, 'c' .. i .. '.txt') end; OUT(tostring(#vim.fn.getcompletion('c', 'buffer')))" },
    { 'wipe-in-bufdelete', "FIX(); vim.api.nvim_create_autocmd('BufDelete', {callback=function(a) pcall(vim.cmd, 'bwipeout! ' .. a.buf) end}); pcall(vim.cmd, 'bdelete 3'); OUT(TRIP(), CUR())" },
    { 'delete-in-bufenter', "FIX(); vim.api.nvim_create_autocmd('BufEnter', {callback=function(a) pcall(vim.cmd, 'bwipeout! ' .. a.buf) end}); pcall(vim.cmd, 'buffer 3'); OUT(TRIP(), CUR())" },
    { 'delete-in-bufunload', "FIX(); vim.api.nvim_create_autocmd('BufUnload', {callback=function() pcall(vim.cmd, 'bwipeout! 5') end}); pcall(vim.cmd, 'bwipeout 3'); OUT(TRIP(), CUR())" },
    { 'bufdo-wipe', "FIX(); pcall(vim.cmd, 'bufdo bwipeout!'); OUT(TRIP(), CUR())" },
    { 'wipe-everything', "FIX(); for i = 1, 20 do pcall(vim.cmd, 'bwipeout! ' .. i) end; OUT(TRIP(), CUR())" },
    { 'fileinfo-longname', "FIX(); pcall(vim.cmd, 'file ' .. string.rep('y', 3000)); local ok, r = pcall(vim.api.nvim_exec2, 'file', {output=true}); OUT(tostring(ok), tostring(ok and #(r.output or '') or r))" },
    { 'ls-after-rename', "FIX(); pcall(vim.cmd, 'buffer 2'); pcall(vim.cmd, 'file ' .. string.rep('z', 2000)); OUT(tostring(#LS('!')))" },
  }
  local aborted = 0
  for _, c in ipairs(CASES) do
    local rc = case('b91/' .. c[1], c[2])
    if rc ~= 0 then
      aborted = aborted + 1
    end
  end
  emit('b91', 'groups', string.format('cases=%d aborted=%d', #CASES, aborted))
end)

emit('##', 'TOTAL', string.format('rows=%d', rows))
structfd:close()
