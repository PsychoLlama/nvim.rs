-- Driver for the autocmd differential sweep; see ausweep.sh.
--
-- Covers what batch B14-15/16 rewrites: autocmd.rs and api/autocmd.rs.
-- The artifact is an **event-order log**: every handler appends one line
-- to `g:AULOG`, and each section prints that list in the order the
-- handlers ran.  A rewrite of the firing walk that keeps every answer
-- and changes only the order is exactly the regression this subsystem
-- produces, and only an ordered artifact sees it.
--
--   §1  order          definition order within one event/pattern
--   §2  patorder       several patterns matching one file
--   §3  buflocal       <buffer> / <buffer=N> against glob patterns
--   §4  patmatch       the pattern matcher: * ** ? {} , \ classes, paths
--   §5  once           ++once: fires once, then is gone
--   §6  nested         nested / ++nested, and the nesting depth limit
--   §7  groups         augroup define / redefine / delete, `autocmd!`
--   §8  groupscope     firing with a group named, :doautocmd <group>
--   §9  selfmod        a handler that edits the autocmd tables *while*
--                      they are being walked -- the au_need_clean
--                      deferred-deletion path, this subsystem's highest
--                      risk shape (AutoPatCmd nodes live on the global
--                      active_apc_list across the firing call)
--   §10 recurse        recursive :doautocmd from inside a handler
--   §11 afile          <afile> / <amatch> / <abuf> / <sfile> / %
--   §12 doautoall      :doautoall and aucmd_prepbuf/aucmd_restbuf
--   §13 eventignore    'eventignore', 'eventignorewin', :noautocmd
--   §14 vevent         v:event for the events that carry one
--   §15 api            nvim_{create,get,del,clear}_autocmd,
--                      nvim_exec_autocmds -- including their error texts
--   §16 listing        :autocmd's own listing (au_show_for_event)
--   §17 exists         exists('#...') and its five arities
--   §18 bufwin         real buffer/window/tab events, in order
--   §19 errors         the error text of a malformed :autocmd
--
-- BLIND SPOTS, deliberately (a half-covered event is worse than a
-- recorded gap):
--   * Anything needing a UI: UIEnter/UILeave, VimResume/VimSuspend,
--     TermResponse, and the tui half of do_autocmd_uienter.
--   * Anything timed: CursorHold/CursorHoldI (updatetime), SafeState and
--     SafeStateAgain (they fire from the main loop's idle point, which a
--     `-l` script never reaches), FocusGained/FocusLost.
--   * Terminal events: --headless -l cannot enter terminal mode at all,
--     so TermOpen/TermEnter/TermLeave/TermClose are unreachable here.
--   * VimEnter/VimLeave*: the script runs after the first and the report
--     is closed before the second.
--   * `:autocmd`'s "Last set from" (verbose listing) resolves against
--     nvim's *starting* directory and is deliberately not recorded.
--   * Docket O-B14-1 (`nvim_set_hl` with a non-positive namespace) has
--     no case here.  Its fix demotes an `assert!` to `debug_assert!`, so
--     the divergence is *release-only*: the debug binary every
--     differential runs still asserts, which is what upstream's default
--     RelWithDebInfo build does too, and a case that aborts the harness
--     gates nothing.  Verified by hand against a release build instead.
--
-- Everything printed has to be reproducible across two builds run
-- minutes apart and from two working directories, so the report carries
-- no address, pid, wall-clock time or path outside the work directory.
--
-- AUSWEEP_ONLY is a Lua pattern matched against each section name; it
-- exists for iterating on one section, not for gating.

local work = assert(os.getenv('AU_WORK'), 'AU_WORK unset')
local structpath = assert(os.getenv('AU_STRUCT'), 'AU_STRUCT unset')
local structfd = assert(io.open(structpath, 'w'))
local only = os.getenv('AUSWEEP_ONLY')
if only == '' then
  only = nil
end

-- Unbuffered: nvim's own messages go to stderr, but a Lua error would
-- otherwise lose the tail of the report.
io.stdout:setvbuf('line')

local function emit(...)
  io.write(table.concat({ ... }, ' '), '\n')
end

local runtime = os.getenv('VIMRUNTIME') or ''
local script = debug.getinfo(1, 'S').source:sub(2)

-- ---------------------------------------------------------------------
-- scrub / esc / canon / errtext are lifted VERBATIM from
-- evalsweep.lua (B14-3).  Not re-derived on purpose: its
-- boolean-key and table-key canon cases were real bugs found the
-- expensive way, and `nvim_get_autocmds` answers reach exactly the same
-- shapes.
-- ---------------------------------------------------------------------

--- Strip the bits of an answer that name where -- or when -- the run
--- happened.  Sorting happens after this, never before: a path is the
--- classic answer that differs between two working directories and
--- survives the three-runs rule.
local function scrub(text)
  text = tostring(text)
  text = text:gsub(vim.pesc(script), '<SCRIPT>')
  text = text:gsub(vim.pesc(work), '<WORK>')
  text = text:gsub(vim.pesc(work:sub(2)), '<WORK>')
  if runtime ~= '' then
    text = text:gsub(vim.pesc(runtime), '<RUNTIME>')
  end
  text = text:gsub('0x%x+', '<ADDR>')
  text = text:gsub('<lambda>%d+', '<lambda>')
  -- A `:autocmd` listing prints a Lua handler as `<Lua 77: <SCRIPT>:1177>`
  -- and, under 'verbose', `Last set from <SCRIPT> line 1175`.  Both are
  -- line numbers *in this file*, so any edit above a section would
  -- otherwise re-baseline the artifact; and the Lua ref is a counter.
  text = text:gsub('<SCRIPT>:%d+', '<SCRIPT>:<LINE>')
  text = text:gsub('<SCRIPT> line %d+', '<SCRIPT> line <LINE>')
  text = text:gsub('<Lua %d+:', '<Lua <REF>:')
  return text
end

--- Escape to one printable line, so a byte difference shows in the diff
--- and a report line stays a report line.
local function esc(bytes)
  return (tostring(bytes):gsub('[^\32-\126]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

local canon

local function canon_number(value)
  if value ~= value then
    return '"nan"'
  elseif value == math.huge then
    return '"inf"'
  elseif value == -math.huge then
    return '"-inf"'
  elseif value == math.floor(value) and math.abs(value) < 2 ^ 53 then
    return string.format('%d', value)
  end
  return string.format('%.17g', value)
end

canon = function(value)
  local kind = type(value)
  if value == vim.NIL then
    return 'null'
  elseif kind == 'nil' then
    return 'nil'
  elseif kind == 'number' then
    return canon_number(value)
  elseif kind == 'boolean' then
    return tostring(value)
  elseif kind == 'string' then
    return '"' .. esc(scrub(value)):gsub('"', '\\"') .. '"'
  elseif kind ~= 'table' then
    return '"<' .. kind .. '>"'
  end
  if next(value) == nil then
    return getmetatable(value) and '{}' or '[]'
  end
  if vim.islist(value) then
    local parts = {}
    for _, item in ipairs(value) do
      parts[#parts + 1] = canon(item)
    end
    return '[' .. table.concat(parts, ',') .. ']'
  end
  local keys, byname = {}, {}
  for key in pairs(value) do
    local name = type(key) == 'string' and esc(key)
      or ('<' .. (type(key) == 'table' and canon(key) or scrub(tostring(key))) .. '>')
    keys[#keys + 1] = name
    byname[name] = key
  end
  table.sort(keys)
  local parts = {}
  for _, name in ipairs(keys) do
    parts[#parts + 1] = '"' .. name .. '":' .. canon(value[byname[name]])
  end
  return '{' .. table.concat(parts, ',') .. '}'
end

local function struct(label, value)
  structfd:write(label, ' ', canon(value), '\n')
end

--- Normalise an error to its message: a pcall against vim.fn or
--- vim.api prefixes the Lua source position, which is a line number in
--- this file and would re-baseline the artifact on any edit.
local function errtext(res)
  local text = scrub(tostring(res))
  text = text:gsub('^.-:%d+: ', '')
  text = text:gsub('^Vim:', '')
  text = text:gsub('^Vim%b():', '')
  return text
end

--- Report a Lua-side answer under a label, both readably and
--- canonically.
local function answer(label, value)
  emit(label, '=', esc(canon(value)))
  struct(label, value)
  return value
end

--- Call something, answering its error text rather than dying.
local function attempt(label, fn, ...)
  local ok, res = pcall(fn, ...)
  if ok then
    return answer(label, res)
  end
  emit(label, '!', esc(errtext(res)))
  struct(label, { err = errtext(res) })
  return nil
end

local SECTIONS = {}
local function section(name, fn)
  SECTIONS[#SECTIONS + 1] = { name = name, fn = fn }
end

-- ---------------------------------------------------------------------
-- The log.  It lives in Vimscript, not in Lua: a Vimscript autocmd body
-- appending to a Lua table would have to go through the lua<->vimscript
-- converter on every single event, and that converter is *evalsweep's*
-- subject.  One conversion per section (reading g:AULOG back) keeps a
-- converter regression out of this artifact.
-- ---------------------------------------------------------------------
vim.cmd([[
  let g:AULOG = []
  function! Au(tag) abort
    call add(g:AULOG, a:tag)
  endfunction
  " Not every event has an <afile>: OptionSet raises E495 for it, and a
  " handler that dies mid-walk answers nothing at all.  Each expansion is
  " therefore asked on its own and a failure is recorded as `!`.
  function! AuExp(what) abort
    try
      return expand(a:what)
    catch
      return '!'
    endtry
  endfunction
  " Same as Au(), but also records what the handler sees of the event:
  " how <afile>/<amatch>/<abuf> and the current buffer/window look from
  " inside the walk rather than after it.
  function! AuCtx(tag) abort
    call add(g:AULOG, a:tag . ' afile=' . AuExp('<afile>')
          \ . ' amatch=' . AuExp('<amatch>') . ' abuf=' . AuExp('<abuf>')
          \ . ' buf=' . bufnr('%') . ' win=' . winnr())
  endfunction
  " For the event-order scenarios: the event's own name has to be passed
  " in, because expand('<event>') answers empty (measured).
  function! AuEv(name) abort
    call add(g:AULOG, a:name . ':' . AuExp('<afile>') . ':' . AuExp('<abuf>')
          \ . ':buf' . bufnr('%') . ':win' . winnr())
  endfunction
]])

--- Run a command, recording its error text rather than dying.  Returns
--- the error, or nil.
local function command(cmd)
  local ok, res = pcall(vim.api.nvim_command, cmd)
  if not ok then
    return errtext(res)
  end
  return nil
end

-- AUSWEEP_TRACE=1 mirrors every step to stderr as it starts.  This
-- subsystem's characteristic failure is a *wedge*, and a wedged run
-- prints nothing at all past the last completed step, so the trace is
-- how you find which one it was.
local trace = os.getenv('AUSWEEP_TRACE') == '1'
local function mark(label)
  if trace then
    io.stderr:write('TRACE ', label, '\n')
  end
end

--- Run a command and report the outcome under `label`; the log is
--- printed separately by `dump`.
local function run(label, cmd)
  mark(label)
  local err = command(cmd)
  if err then
    emit(label, '!', esc(err))
    struct(label .. '/err', err)
  end
  return err
end

--- Create an augroup with `:augroup`, which is two commands: `:autocmd
--- {group} ...` refuses a group that does not exist yet (E216), so every
--- section that names a group has to declare it first.  Deliberately the
--- ex-command path rather than nvim_create_augroup -- §15 asks the API.
local function mkgroup(name)
  vim.cmd('silent! augroup ' .. name)
  vim.cmd('silent! augroup END')
end

--- Autocmd and augroup ids are one monotonic counter shared by the whole
--- run, so recording a raw one makes every later section a function of
--- how many ids the earlier ones allocated.  What is worth asserting is
--- that an id was handed out and whether two calls agree, not its value.
local function idof(value)
  return type(value) == 'number' and '<id>' or value
end

--- Print the event-order log, in order, and clear it.
local function dump(label)
  local log = vim.g.AULOG
  for i, entry in ipairs(log) do
    emit(label, string.format('%02d', i), esc(scrub(entry)))
  end
  if #log == 0 then
    emit(label, '--', '(no handler ran)')
  end
  struct(label, log)
  vim.cmd('let g:AULOG = []')
end

--- Back to a fixed state: no autocmds anywhere, one empty buffer, one
--- window, the event options cleared.  `noautocmd` throughout, so the
--- teardown of one section cannot write into the log of the next.
---
--- It *clears* every group rather than deleting it, and that is not
--- tidiness.  `:augroup! X` leaves the id behind under the name
--- `--Deleted--`, which `getcompletion('', 'augroup')` then answers
--- with; `:augroup --Deleted--` creates a real group of that name, and
--- deleting *it* leaves another marker.  Deleting what the completion
--- offers therefore doubles the group table on every reset -- measured:
--- 650,000 trace lines and a wedged run.  §7 asks about `:augroup!`
--- deliberately; nothing else may.
local function reset()
  mark('reset/opts')
  vim.cmd('silent! noautocmd set eventignore= eventignorewin=')
  vim.cmd('silent! noautocmd autocmd!')
  for _, name in ipairs(vim.fn.getcompletion('', 'augroup')) do
    if name ~= 'END' and name ~= '--Deleted--' then
      vim.cmd(('silent! noautocmd autocmd! %s'):format(name))
    end
  end
  mark('reset/win')
  vim.cmd('silent! noautocmd only!')
  vim.cmd('silent! noautocmd tabonly!')
  vim.cmd('silent! noautocmd enew!')
  vim.cmd('silent! noautocmd %bwipeout!')
  mark('reset/done')
  vim.cmd('let g:AULOG = []')
end

-- The sweep runs from the fixture tree, so a relative `<afile>` is a
-- bare file name and the report does not depend on how deep the work
-- directory is.
vim.cmd('silent! noautocmd cd ' .. vim.fn.fnameescape(work .. '/tree'))
vim.cmd([[
  silent! set noswapfile nobackup nowritebackup noundofile undolevels=-1
  silent! set shortmess=filnxtToOF report=9999 nomore
  silent! set shada= viewdir=/dev/null nofsync
  silent! filetype off
  silent! syntax off
]])

-- ---------------------------------------------------------------------
-- §1  Firing order within one event
-- ---------------------------------------------------------------------
section('order', function()
  reset()
  -- Five handlers on one event and one pattern, spread over the default
  -- group and two named ones.  Upstream fires them in *definition*
  -- order regardless of group, because a group is a filter on the walk,
  -- not a separate list.
  run('order/def', 'autocmd User AuOrder call Au("1-default")')
  mkgroup('G1')
  mkgroup('G2')
  run('order/g1', 'autocmd G1 User AuOrder call Au("2-G1")')
  run('order/def2', 'autocmd User AuOrder call Au("3-default")')
  run('order/g2', 'autocmd G2 User AuOrder call Au("4-G2")')
  run('order/g1b', 'autocmd G1 User AuOrder call Au("5-G1")')
  run('order/fire', 'doautocmd User AuOrder')
  dump('order/log')

  -- Re-registering the *same* command in the same group and pattern is
  -- not a no-op: upstream appends, so the handler runs twice.
  reset()
  run('order/dup1', 'autocmd User AuDup call Au("dup")')
  run('order/dup2', 'autocmd User AuDup call Au("dup")')
  run('order/dupfire', 'doautocmd User AuDup')
  dump('order/duplog')

  -- Two events, one :doautocmd each, to pin that the per-event lists are
  -- independent and that the walk starts at the head of each.
  reset()
  run('order/e1', 'autocmd User AuA call Au("A1")')
  run('order/e2', 'autocmd User AuB call Au("B1")')
  run('order/e3', 'autocmd User AuA call Au("A2")')
  run('order/fireb', 'doautocmd User AuB')
  run('order/firea', 'doautocmd User AuA')
  dump('order/twolog')
end)

-- ---------------------------------------------------------------------
-- §2  Several patterns matching one file
-- ---------------------------------------------------------------------
section('patorder', function()
  reset()
  -- All six patterns match `one.txt`.  The order they fire in is the
  -- order the *patterns* were defined, which is what aucmd_next walks;
  -- an implementation that grouped by pattern shape, or that matched
  -- the most specific first, would reorder this list.
  run('patorder/p1', 'autocmd BufNewFile * call Au("1-star")')
  run('patorder/p2', 'autocmd BufNewFile *.txt call Au("2-startxt")')
  run('patorder/p3', 'autocmd BufNewFile one.txt call Au("3-exact")')
  run('patorder/p4', 'autocmd BufNewFile ?ne.txt call Au("4-question")')
  run('patorder/p5', 'autocmd BufNewFile {one,two}.txt call Au("5-brace")')
  run('patorder/p6', 'autocmd BufNewFile *.log,one.txt call Au("6-commalist")')
  run('patorder/fire', 'doautocmd BufNewFile one.txt')
  dump('patorder/log')

  -- The same set fired against a name only some of them match.
  run('patorder/fire2', 'doautocmd BufNewFile two.txt')
  dump('patorder/log2')
  run('patorder/fire3', 'doautocmd BufNewFile three.log')
  dump('patorder/log3')

  -- Interleaving two events over the same patterns: the per-event lists
  -- must not share a walk cursor.
  reset()
  run('patorder/x1', 'autocmd BufNewFile *.txt call Au("txt-new")')
  run('patorder/x2', 'autocmd BufReadPre *.txt call Au("txt-pre")')
  run('patorder/x3', 'autocmd BufNewFile * call Au("any-new")')
  run('patorder/xfire', 'doautocmd BufNewFile one.txt')
  dump('patorder/xlog')
end)

-- ---------------------------------------------------------------------
-- §3  Buffer-local patterns against globs
-- ---------------------------------------------------------------------
section('buflocal', function()
  reset()
  run('buflocal/open', 'edit one.txt')
  local bufnr = vim.fn.bufnr('%')
  answer('buflocal/bufnr', bufnr)
  vim.cmd('let g:AULOG = []')
  -- NOT a User event: for User the pattern position holds the user
  -- event's *name*, so `<buffer>` there is parsed as part of the
  -- command and answers E488 (measured).  BufReadPre is a file event
  -- whose pattern is matched against the buffer, which is the question.
  --
  -- A <buffer> pattern is stored as the literal `<buffer=N>`, so it
  -- sorts into the same per-event list as a glob and fires in
  -- definition order with them -- it is not a separate, higher-priority
  -- list.  Interleave the two to prove which.
  run('buflocal/a', 'autocmd BufReadPre * call Au("1-glob")')
  run('buflocal/b', 'autocmd BufReadPre <buffer> call Au("2-buffer")')
  run('buflocal/c', 'autocmd BufReadPre one.txt call Au("3-glob")')
  run('buflocal/d', ('autocmd BufReadPre <buffer=%d> call Au("4-buffer-n")'):format(bufnr))
  run('buflocal/fire', 'doautocmd BufReadPre')
  dump('buflocal/log')

  -- The same event fired while a *different* buffer is current: the
  -- buffer-local handlers must not run, and the name-matched one must
  -- stop matching too.
  run('buflocal/other', 'noautocmd edit two.txt')
  run('buflocal/fire2', 'doautocmd BufReadPre')
  dump('buflocal/log2')
  -- ... and named explicitly, so the pattern matches the argument while
  -- the buffer is still the other one.
  run('buflocal/fire3', 'doautocmd BufReadPre one.txt')
  dump('buflocal/log3')

  -- <buffer=N> for a buffer that does not exist, and for one that is
  -- wiped afterwards (aubuflocal_remove turns the pattern into
  -- <buffer=0>, so the entry survives the wipe and never matches again).
  answer('buflocal/badnr', command('autocmd BufReadPre <buffer=99999> call Au("ghost")'))
  local function scrubbed(list)
    for _, entry in ipairs(list or {}) do
      entry.id, entry.group = idof(entry.id), idof(entry.group)
    end
    return list
  end
  answer('buflocal/list', scrubbed(vim.api.nvim_get_autocmds({ event = 'BufReadPre' })))
  run('buflocal/wipe', ('noautocmd bwipeout! %d'):format(bufnr))
  answer('buflocal/after-wipe', scrubbed(vim.api.nvim_get_autocmds({ event = 'BufReadPre' })))
  run('buflocal/fire4', 'doautocmd BufReadPre one.txt')
  dump('buflocal/log4')
end)

-- ---------------------------------------------------------------------
-- §4  The pattern matcher
-- ---------------------------------------------------------------------
section('patmatch', function()
  -- One autocmd per pattern, all on one event, each logging its own
  -- index; then the event is fired once per file name.  The log for a
  -- name is therefore the *set* of patterns that matched it, in
  -- definition order -- one artifact covering the whole matrix.
  local PATTERNS = {
    '*',
    '*.txt',
    '*.c',
    '?.c',
    '??.c',
    'one.txt',
    'one*',
    '*one*',
    '{one,two}.txt',
    '{a,ab}.c',
    'one.txt,two.txt',
    '*.txt,*.c',
    '**/one.txt',
    '**/*.txt',
    'sub/*.txt',
    'sub/**',
    '*/one.txt',
    'UPPER.TXT',
    'upper.txt',
    'no_ext',
    'no\\_ext',
    '\\<one\\>',
    'x,y.txt',
    'x\\,y.txt',
    '[ao]*.c',
    '*[0-9]*',
    'back\\\\slash.txt',
    'q\\?mark.txt',
    'br\\{ace}.txt',
    work .. '/tree/one.txt',
    '*/tree/one.txt',
  }
  local NAMES = {
    'one.txt',
    'two.txt',
    'three.log',
    'a.c',
    'ab.c',
    'no_ext',
    'UPPER.TXT',
    'sub/one.txt',
    'sub/deep/one.txt',
    work .. '/tree/one.txt',
    work .. '/other/one.txt',
    'x,y.txt',
  }
  -- Defined through nvim_create_autocmd, which takes the pattern
  -- literally: that asks about the *matcher* rather than about
  -- `:autocmd`'s own argument splitting, which §19 asks separately.
  reset()
  for i, pat in ipairs(PATTERNS) do
    local ok, err = pcall(vim.api.nvim_create_autocmd, 'User', {
      pattern = pat,
      command = ('call Au("p%02d")'):format(i),
    })
    if not ok then
      emit('patmatch/def', string.format('p%02d %s', i, esc(pat)), '!', esc(errtext(err)))
    end
  end
  for _, pat in ipairs(PATTERNS) do
    emit('patmatch/pattern', esc(pat))
  end
  for _, name in ipairs(NAMES) do
    -- `:doautocmd {event} {fname}` matches the *given* name against
    -- every pattern (a User event has no buffer of its own), which is
    -- exactly aucmd_next's question with nothing else mixed in.
    run('patmatch/fire', 'doautocmd User ' .. vim.fn.fnameescape(name))
    dump('patmatch/' .. scrub(name))
  end
end)

-- ---------------------------------------------------------------------
-- §5  ++once
-- ---------------------------------------------------------------------
section('once', function()
  reset()
  run('once/a', 'autocmd User AuOnce call Au("plain-1")')
  run('once/b', 'autocmd User AuOnce ++once call Au("once-2")')
  run('once/c', 'autocmd User AuOnce call Au("plain-3")')
  run('once/d', 'autocmd User AuOnce ++once call Au("once-4")')
  run('once/fire1', 'doautocmd User AuOnce')
  dump('once/log1')
  run('once/fire2', 'doautocmd User AuOnce')
  dump('once/log2')
  attempt('once/left', vim.api.nvim_get_autocmds, { event = 'User' })

  -- ++once removal happens *inside* the walk, so the handler that runs
  -- after a ++once one has to survive the deletion.  A ++once handler
  -- that itself fires the same event is the sharp version.
  reset()
  run('once/n1', 'autocmd User AuOnce2 ++once call Au("once-a")')
  run('once/n2', 'autocmd User AuOnce2 call Au("plain-b")')
  run('once/nfire', 'doautocmd User AuOnce2')
  dump('once/nlog')

  -- ++once inside a group, and the group listing afterwards.
  reset()
  mkgroup('GOnce')
  run('once/g', 'autocmd GOnce User AuOnce3 ++once call Au("g-once")')
  run('once/gfire1', 'doautocmd User AuOnce3')
  run('once/gfire2', 'doautocmd User AuOnce3')
  dump('once/glog')
  attempt('once/gleft', vim.api.nvim_get_autocmds, { event = 'User' })
end)

-- ---------------------------------------------------------------------
-- §6  nested / ++nested and the depth limit
-- ---------------------------------------------------------------------
section('nested', function()
  -- Without `nested`, a handler's own :doautocmd of *another* event does
  -- fire (:doautocmd is explicit), but the implicit events raised by a
  -- command inside the handler do not.  `:edit` is the classic case.
  reset()
  run('nested/a', 'autocmd User AuNest call Au("outer") | silent! edit one.txt')
  run('nested/b', 'autocmd BufReadPre one.txt call Au("inner-nonested")')
  run('nested/fire', 'doautocmd User AuNest')
  dump('nested/log-plain')

  reset()
  run('nested/c', 'autocmd User AuNest2 nested call Au("outer") | silent! edit two.txt')
  run('nested/d', 'autocmd BufReadPre two.txt call Au("inner-nested")')
  run('nested/fire2', 'doautocmd User AuNest2')
  dump('nested/log-nested')

  -- ++nested is the same flag by its modern spelling; both must produce
  -- the same log.
  reset()
  run('nested/e', 'autocmd User AuNest3 ++nested call Au("outer") | silent! edit three.log')
  run('nested/f', 'autocmd BufReadPre three.log call Au("inner-plusnested")')
  run('nested/fire3', 'doautocmd User AuNest3')
  dump('nested/log-plusnested')

  -- The depth limit: a nested handler that re-raises its own event.
  -- Upstream stops at ten with E218 rather than recursing forever, and
  -- the count of log lines is the assertion.
  reset()
  run('nested/deep', 'autocmd User AuDeep nested call Au("deep") | doautocmd User AuDeep')
  run('nested/deepfire', 'doautocmd User AuDeep')
  local log = vim.g.AULOG
  answer('nested/deepcount', #log)
  vim.cmd('let g:AULOG = []')

  -- Without nested the same shape still recurses, because :doautocmd is
  -- explicit -- the limit is the same one.
  reset()
  run('nested/deep2', 'autocmd User AuDeep2 call Au("deep2") | doautocmd User AuDeep2')
  run('nested/deep2fire', 'doautocmd User AuDeep2')
  answer('nested/deep2count', #vim.g.AULOG)
  vim.cmd('let g:AULOG = []')
end)

-- ---------------------------------------------------------------------
-- §7  Groups
-- ---------------------------------------------------------------------
section('groups', function()
  reset()
  mkgroup('GA')
  mkgroup('GB')
  run('groups/def', 'autocmd GA User AuG call Au("GA-1")')
  run('groups/def2', 'autocmd GB User AuG call Au("GB-1")')
  run('groups/def3', 'autocmd GA User AuG call Au("GA-2")')
  run('groups/fire', 'doautocmd User AuG')
  dump('groups/log')

  -- `augroup GA | autocmd!` clears only GA's autocmds; GB's survive and
  -- keep their relative order with anything defined afterwards.
  run('groups/clear', 'autocmd! GA')
  run('groups/fire2', 'doautocmd User AuG')
  dump('groups/log2')
  run('groups/re', 'autocmd GA User AuG call Au("GA-3")')
  run('groups/fire3', 'doautocmd User AuG')
  dump('groups/log3')

  -- `autocmd! {event}` and `autocmd! * {pat}` inside a group.
  reset()
  run('groups/m1', 'augroup GC')
  run('groups/m2', 'autocmd User AuG1 call Au("g1")')
  run('groups/m3', 'autocmd User AuG2 call Au("g2")')
  run('groups/m4', 'autocmd BufNewFile *.txt call Au("bnf")')
  run('groups/m5', 'augroup END')
  run('groups/m6', 'autocmd! GC User AuG1')
  run('groups/f1', 'doautocmd User AuG1')
  run('groups/f2', 'doautocmd User AuG2')
  run('groups/f3', 'doautocmd BufNewFile one.txt')
  dump('groups/mlog')

  -- Deleting a group: `augroup! X`.  Deleting one that still holds
  -- autocmds is an error upstream reports rather than a silent drop.
  --
  -- Its autocmds SURVIVE, under the group name `--Deleted--`, and
  -- nothing can reach them again: no group-scoped `:autocmd!` (the name
  -- no longer resolves) and not `nvim_clear_autocmds` either (measured
  -- below).  The event here is deliberately BufWritePost with a pattern
  -- nothing else uses, so the survivor is inert for every later section
  -- but still visible in §16's full listing, which is where it belongs.
  reset()
  mkgroup('GD')
  run('groups/d1', 'autocmd GD BufWritePost zz-deleted-group-leak call Au("gd")')
  run('groups/d2', 'augroup! GD')
  run('groups/d3', 'doautocmd BufWritePost zz-deleted-group-leak')
  dump('groups/dlog')
  run('groups/d4', 'autocmd! GD')
  run('groups/d5', 'augroup! GD')
  answer('groups/d6', vim.fn.exists('#GD'))
  -- Deleting a group that does not exist, and the one nvim never lets
  -- you delete.
  run('groups/d7', 'augroup! NoSuchGroup')
  run('groups/d8', 'augroup! end')
  -- `:augroup! X` on a group that still holds autocmds keeps them, under
  -- the name `--Deleted--`, and no group-scoped `:autocmd!` can reach
  -- them again -- they would leak into every later section's listing.
  -- Clearing them by pattern is the only way back.
  answer(
    'groups/d9',
    vim.api.nvim_clear_autocmds({
      event = 'BufWritePost',
      pattern = 'zz-deleted-group-leak',
    })
  )
  answer(
    'groups/d10',
    vim.api.nvim_get_autocmds({
      event = 'BufWritePost',
      pattern = 'zz-deleted-group-leak',
    })
  )

  -- Group ids and names through the API.
  reset()
  local id = vim.api.nvim_create_augroup('GE', { clear = true })
  local id2 = vim.api.nvim_create_augroup('GE', { clear = false })
  -- The value is a counter shared by the whole run; what is asked here
  -- is that a second create of the same name answers the same id.
  answer('groups/api-id', { got = idof(id), same = id == id2 })
  answer('groups/api-au', idof(vim.api.nvim_create_autocmd('User', {
    group = id,
    pattern = 'AuE',
    command = 'call Au("ge")',
  })))
  run('groups/api-fire', 'doautocmd User AuE')
  dump('groups/api-log')
  attempt('groups/api-del', vim.api.nvim_del_augroup_by_id, id)
  attempt('groups/api-del2', vim.api.nvim_del_augroup_by_id, id)
  attempt('groups/api-delname', vim.api.nvim_del_augroup_by_name, 'NoSuchGroup')
end)

-- ---------------------------------------------------------------------
-- §8  Firing with a group named
-- ---------------------------------------------------------------------
section('groupscope', function()
  reset()
  run('groupscope/a', 'autocmd User AuS call Au("default")')
  mkgroup('GS1')
  mkgroup('GS2')
  run('groupscope/b', 'autocmd GS1 User AuS call Au("GS1")')
  run('groupscope/c', 'autocmd GS2 User AuS call Au("GS2")')
  run('groupscope/all', 'doautocmd User AuS')
  dump('groupscope/log-all')
  run('groupscope/one', 'doautocmd GS1 User AuS')
  dump('groupscope/log-gs1')
  run('groupscope/two', 'doautocmd GS2 User AuS')
  dump('groupscope/log-gs2')
  run('groupscope/miss', 'doautocmd NoSuchGroup User AuS')
  dump('groupscope/log-miss')
  attempt('groupscope/api', vim.api.nvim_exec_autocmds, 'User', { pattern = 'AuS', group = 'GS1' })
  dump('groupscope/log-api')
  attempt('groupscope/api-bad', vim.api.nvim_exec_autocmds, 'User', {
    pattern = 'AuS',
    group = 'NoSuchGroup',
  })
  dump('groupscope/log-apibad')
end)

-- ---------------------------------------------------------------------
-- §9  Self-modifying handlers -- the au_need_clean path
-- ---------------------------------------------------------------------
section('selfmod', function()
  -- (a) A handler that clears every autocmd for the event it is
  -- currently being fired for.  The walk is live: AutoPatCmd is a stack
  -- node on active_apc_list, and au_cleanup must defer the free until
  -- the walk unwinds.  The handlers defined *after* the deleter are the
  -- assertion -- upstream does not run them.
  reset()
  run('selfmod/a1', 'autocmd User AuSM call Au("1")')
  run('selfmod/a2', 'autocmd User AuSM call Au("2-deleter") | autocmd! User AuSM')
  run('selfmod/a3', 'autocmd User AuSM call Au("3")')
  run('selfmod/a4', 'autocmd User AuSM call Au("4")')
  run('selfmod/afire', 'doautocmd User AuSM')
  dump('selfmod/log-clear')
  run('selfmod/afire2', 'doautocmd User AuSM')
  dump('selfmod/log-clear2')

  -- (b) The same, from inside a group, clearing only its own group.
  reset()
  mkgroup('GSM')
  run('selfmod/b1', 'autocmd User AuSM2 call Au("default-1")')
  run(
    'selfmod/b2',
    'autocmd GSM User AuSM2 call Au("g-2") | doautocmd User AuNothing'
  )
  run('selfmod/b3', 'autocmd GSM User AuSM2 call Au("g-3-deleter") | autocmd! GSM')
  run('selfmod/b4', 'autocmd GSM User AuSM2 call Au("g-4")')
  run('selfmod/b5', 'autocmd User AuSM2 call Au("default-5")')
  run('selfmod/bfire', 'doautocmd User AuSM2')
  dump('selfmod/log-group')

  -- (c) A handler that *adds* autocmds for the event being fired.  A
  -- walk that re-read the list head would run the new ones; upstream
  -- does not, because aucmd_next has already passed them.
  reset()
  run('selfmod/c1', 'autocmd User AuSM3 call Au("1")')
  run('selfmod/c2', 'autocmd User AuSM3 call Au("2-adder") '
    .. '| autocmd User AuSM3 call Au("added-late")')
  run('selfmod/c3', 'autocmd User AuSM3 call Au("3")')
  run('selfmod/cfire', 'doautocmd User AuSM3')
  dump('selfmod/log-add')
  run('selfmod/cfire2', 'doautocmd User AuSM3')
  dump('selfmod/log-add2')

  -- (d) A handler that deletes exactly *one* later handler by pattern.
  reset()
  run('selfmod/d1', 'autocmd User AuSM4 call Au("1")')
  run('selfmod/d2', 'autocmd User AuSM4 call Au("2-deleter") '
    .. '| autocmd! User AuSM4x')
  run('selfmod/d3', 'autocmd User AuSM4x call Au("x-victim")')
  run('selfmod/d4', 'autocmd User AuSM4 call Au("3")')
  run('selfmod/dfire', 'doautocmd User AuSM4')
  dump('selfmod/log-one')
  run('selfmod/dfire2', 'doautocmd User AuSM4x')
  dump('selfmod/log-onex')

  -- (e) A ++once handler that deletes the whole event, so the deferred
  -- free and the ++once removal land on the same walk.
  reset()
  run('selfmod/e1', 'autocmd User AuSM5 call Au("1")')
  run('selfmod/e2', 'autocmd User AuSM5 ++once call Au("2-once-deleter") '
    .. '| autocmd! User AuSM5')
  run('selfmod/e3', 'autocmd User AuSM5 call Au("3")')
  run('selfmod/efire', 'doautocmd User AuSM5')
  dump('selfmod/log-oncedel')
  attempt('selfmod/eleft', vim.api.nvim_get_autocmds, { event = 'User' })

  -- (f) Deletion during a *nested* walk: the inner :doautocmd clears the
  -- outer event while the outer walk is still on the stack.  This is the
  -- shape that puts two AutoPatCmd nodes on active_apc_list and deletes
  -- an AutoPat the outer one still points at.
  reset()
  run('selfmod/f1', 'autocmd User AuOut call Au("out-1")')
  run('selfmod/f2', 'autocmd User AuOut call Au("out-2") | doautocmd User AuIn')
  run('selfmod/f3', 'autocmd User AuOut call Au("out-3")')
  run('selfmod/f4', 'autocmd User AuIn call Au("in-1") | autocmd! User AuOut')
  run('selfmod/ffire', 'doautocmd User AuOut')
  dump('selfmod/log-nesteddel')
  attempt('selfmod/fleft', vim.api.nvim_get_autocmds, { event = 'User' })

  -- (g) The API deleter: nvim_del_autocmd by id from inside the walk.
  reset()
  local ids = {}
  for i = 1, 4 do
    ids[i] = vim.api.nvim_create_autocmd('User', {
      pattern = 'AuSM6',
      command = ('call Au("api-%d")'):format(i),
    })
  end
  vim.api.nvim_create_autocmd('User', {
    pattern = 'AuSM6',
    callback = function()
      vim.fn.Au('api-killer')
      pcall(vim.api.nvim_del_autocmd, ids[4])
    end,
  })
  -- The killer is defined last, so at the time it runs the victim has
  -- already fired; fire twice and diff the two logs.
  run('selfmod/gfire1', 'doautocmd User AuSM6')
  dump('selfmod/log-api1')
  run('selfmod/gfire2', 'doautocmd User AuSM6')
  dump('selfmod/log-api2')
end)

-- ---------------------------------------------------------------------
-- §10  Recursive :doautocmd
-- ---------------------------------------------------------------------
section('recurse', function()
  reset()
  run('recurse/a1', 'autocmd User AuR1 call Au("r1-a") | doautocmd User AuR2')
  run('recurse/a2', 'autocmd User AuR1 call Au("r1-b")')
  run('recurse/a3', 'autocmd User AuR2 call Au("r2-a") | doautocmd User AuR3')
  run('recurse/a4', 'autocmd User AuR3 call Au("r3-a")')
  run('recurse/afire', 'doautocmd User AuR1')
  dump('recurse/log')

  -- The same event re-entered with a different pattern: one AutoPatCmd
  -- per level over the *same* AutoPat list.
  reset()
  run('recurse/b1', 'autocmd User AuX call Au("x-1") | doautocmd User AuY')
  run('recurse/b2', 'autocmd User AuY call Au("y-1")')
  run('recurse/b3', 'autocmd User Au* call Au("star")')
  run('recurse/bfire', 'doautocmd User AuX')
  dump('recurse/log2')

  -- A handler that fires an event with no handlers at all, then one
  -- that fires an unknown event name.
  reset()
  run('recurse/c1', 'autocmd User AuZ call Au("z") | doautocmd User AuNobody')
  run('recurse/cfire', 'doautocmd User AuZ')
  dump('recurse/log3')
  run('recurse/bad', 'doautocmd NoSuchEvent')
  run('recurse/bad2', 'doautocmd User')
end)

-- ---------------------------------------------------------------------
-- §11  <afile> / <amatch> / <abuf>
-- ---------------------------------------------------------------------
section('afile', function()
  reset()
  run('afile/a', 'autocmd User * call AuCtx("user")')
  run('afile/f1', 'doautocmd User AuThing')
  dump('afile/log-user')

  reset()
  run('afile/b', 'autocmd BufNewFile,BufReadPre * call AuCtx("buf")')
  run('afile/f2', 'doautocmd BufNewFile sub/one.txt')
  dump('afile/log-rel')
  run('afile/f3', 'doautocmd BufReadPre ' .. vim.fn.fnameescape(work .. '/tree/one.txt'))
  dump('afile/log-abs')

  -- A real :edit, where <afile>/<abuf> come from the buffer rather than
  -- from the :doautocmd argument.
  reset()
  run('afile/c', 'autocmd BufNewFile,BufReadPost,BufEnter * call AuCtx("real")')
  run('afile/f4', 'edit lines.txt')
  dump('afile/log-edit')
  run('afile/f5', 'edit fresh_file.txt')
  dump('afile/log-new')

  -- <amatch> differs from <afile> for the events whose pattern is
  -- matched against something other than the file name: FileType,
  -- Syntax, User, OptionSet, CmdUndefined.
  reset()
  run('afile/d', 'autocmd FileType * call AuCtx("filetype")')
  run('afile/f6', 'setfiletype lua')
  dump('afile/log-filetype')

  reset()
  run('afile/e', 'autocmd OptionSet number call AuCtx("optionset")')
  run('afile/f7', 'setlocal number')
  dump('afile/log-optionset')
end)

-- ---------------------------------------------------------------------
-- §12  :doautoall, aucmd_prepbuf / aucmd_restbuf
-- ---------------------------------------------------------------------
section('doautoall', function()
  reset()
  run('doautoall/o1', 'noautocmd edit one.txt')
  run('doautoall/o2', 'noautocmd edit two.txt')
  run('doautoall/o3', 'noautocmd edit three.log')
  answer('doautoall/current', vim.fn.bufnr('%'))
  vim.cmd('let g:AULOG = []')
  -- aucmd_prepbuf either switches the current window to the buffer or
  -- opens the hidden autocmd window; AuCtx records buf and win, so the
  -- log says which happened for each buffer.
  run('doautoall/a', 'autocmd User AuAll call AuCtx("all")')
  run('doautoall/fire', 'doautoall User AuAll')
  dump('doautoall/log')
  answer('doautoall/after', vim.fn.bufnr('%'))
  answer('doautoall/wins', vim.fn.winnr('$'))

  -- With a group named, and with a pattern.
  reset()
  run('doautoall/g1', 'noautocmd edit one.txt')
  run('doautoall/g2', 'noautocmd edit two.txt')
  mkgroup('GAll')
  run('doautoall/g3', 'autocmd GAll User AuAll2 call AuCtx("g")')
  run('doautoall/g4', 'autocmd User AuAll2 call AuCtx("d")')
  run('doautoall/gfire', 'doautoall GAll User AuAll2')
  dump('doautoall/glog')

  -- is_aucmd_win: does the handler see the autocmd window?
  reset()
  run('doautoall/w1', 'noautocmd edit one.txt')
  run('doautoall/w2', 'noautocmd edit two.txt')
  vim.api.nvim_create_autocmd('User', {
    pattern = 'AuWin',
    callback = function()
      local win = vim.api.nvim_get_current_win()
      vim.fn.Au(
        ('aucmd_win=%s bufs=%d wins=%d'):format(
          tostring(vim.fn.win_gettype(win)),
          #vim.api.nvim_list_bufs(),
          #vim.api.nvim_list_wins()
        )
      )
    end,
  })
  run('doautoall/wfire', 'doautoall User AuWin')
  dump('doautoall/wlog')

  -- :doautoall on a buffer-local pattern, which prepbuf has to make
  -- current before the pattern can match at all.
  reset()
  run('doautoall/b1', 'noautocmd edit one.txt')
  local b1 = vim.fn.bufnr('%')
  run('doautoall/b2', 'noautocmd edit two.txt')
  run('doautoall/b3', ('autocmd BufReadPre <buffer=%d> call AuCtx("bl")'):format(b1))
  run('doautoall/b4', 'autocmd BufReadPre * call AuCtx("gl")')
  run('doautoall/bfire', 'doautoall BufReadPre')
  dump('doautoall/blog')
end)

-- ---------------------------------------------------------------------
-- §13  'eventignore' and :noautocmd
-- ---------------------------------------------------------------------
section('eventignore', function()
  reset()
  run('eventignore/a', 'autocmd User AuEI call Au("ei")')
  run('eventignore/b', 'autocmd BufNewFile * call Au("bnf")')
  run('eventignore/f1', 'doautocmd User AuEI')
  dump('eventignore/log-none')

  run('eventignore/set1', 'set eventignore=User')
  run('eventignore/f2', 'doautocmd User AuEI')
  run('eventignore/f3', 'doautocmd BufNewFile one.txt')
  dump('eventignore/log-user')

  run('eventignore/set2', 'set eventignore=all')
  run('eventignore/f4', 'doautocmd User AuEI')
  run('eventignore/f5', 'doautocmd BufNewFile one.txt')
  dump('eventignore/log-all')

  -- `all,-Event` is the subtractive form.
  run('eventignore/set3', 'set eventignore=all,-User')
  run('eventignore/f6', 'doautocmd User AuEI')
  run('eventignore/f7', 'doautocmd BufNewFile one.txt')
  dump('eventignore/log-allbut')

  run('eventignore/set4', 'set eventignore=')
  run('eventignore/f8', 'doautocmd User AuEI')
  dump('eventignore/log-cleared')

  -- check_ei rejects an unknown name and leaves the old value.
  run('eventignore/bad', 'set eventignore=NoSuchEvent')
  answer('eventignore/value', vim.api.nvim_get_option_value('eventignore', {}))

  -- :noautocmd suppresses everything for the duration of one command.
  reset()
  run('eventignore/n1', 'autocmd BufNewFile,BufReadPre,BufEnter * call Au("n")')
  run('eventignore/n2', 'noautocmd edit one.txt')
  dump('eventignore/log-noautocmd')
  run('eventignore/n3', 'edit two.txt')
  dump('eventignore/log-autocmd')

  -- 'eventignorewin' is per-window and only covers the window and buffer
  -- events that have a window to be attributed to.  The control run
  -- first, then the same scenario with the option set: a section whose
  -- two halves answer the same thing is a hole, not coverage.
  reset()
  run('eventignore/w0', 'noautocmd edit one.txt')
  run('eventignore/w1', 'autocmd BufEnter,BufLeave,WinEnter,WinLeave,WinNew * call AuEv("W")')
  run('eventignore/w2', 'split two.txt')
  dump('eventignore/log-eiw-control')
  run('eventignore/w3', 'noautocmd only')
  run('eventignore/w4', 'setlocal eventignorewin=WinEnter,WinLeave')
  run('eventignore/w5', 'split two.txt')
  dump('eventignore/log-eiw')
  run('eventignore/w6', 'setlocal eventignorewin=all')
  run('eventignore/w7', 'wincmd w')
  dump('eventignore/log-eiw-all')
  run('eventignore/w8', 'setlocal eventignorewin=')
end)

-- ---------------------------------------------------------------------
-- §14  v:event
-- ---------------------------------------------------------------------
section('vevent', function()
  local function veventof(label, setup, trigger, prep)
    reset()
    vim.cmd('let g:AUEV = {}')
    if prep then
      prep()
    end
    run(label .. '/def', setup)
    run(label .. '/fire', trigger)
    attempt(label, vim.api.nvim_eval, 'g:AUEV')
    dump(label .. '/log')
  end

  veventof(
    'vevent/yank',
    'autocmd TextYankPost * let g:AUEV = deepcopy(v:event) | call Au("yank")',
    'normal! yyjyw',
    function()
      vim.api.nvim_buf_set_lines(0, 0, -1, false, { 'alpha beta', 'gamma delta' })
    end
  )
  veventof(
    'vevent/dirchanged',
    'autocmd DirChanged * let g:AUEV = deepcopy(v:event) | call AuCtx("dir")',
    'cd ' .. vim.fn.fnameescape(work .. '/other')
  )
  vim.cmd('silent! noautocmd cd ' .. vim.fn.fnameescape(work .. '/tree'))
  veventof(
    'vevent/dirchangedpre',
    'autocmd DirChangedPre * let g:AUEV = deepcopy(v:event) | call AuCtx("dirpre")',
    'lcd ' .. vim.fn.fnameescape(work .. '/other')
  )
  vim.cmd('silent! noautocmd cd ' .. vim.fn.fnameescape(work .. '/tree'))
  veventof(
    'vevent/recording',
    'autocmd RecordingEnter,RecordingLeave * let g:AUEV = deepcopy(v:event) | call Au("rec")',
    'normal! qaiz\27q'
  )

  -- OptionSet does not use v:event at all: it carries v:option_old,
  -- v:option_new, v:option_type, v:option_command, v:option_oldlocal and
  -- v:option_oldglobal instead (measured -- v:event is an empty dict
  -- there).  Asked separately, local and global, because the two fill a
  -- different subset.
  local OPTVARS = 'v:option_type . "|" . v:option_command . "|" . v:option_old'
    .. ' . "|" . v:option_new . "|" . v:option_oldlocal . "|" . v:option_oldglobal'
  reset()
  run('vevent/opt-def', 'autocmd OptionSet number let g:AUEV = ' .. OPTVARS
    .. ' | call Au("opt")')
  run('vevent/opt-fire', 'setlocal number')
  attempt('vevent/optionset-local', vim.api.nvim_eval, 'g:AUEV')
  dump('vevent/optionset-local/log')
  reset()
  run('vevent/optg-def', 'autocmd OptionSet wrap let g:AUEV = ' .. OPTVARS
    .. ' | call Au("optg")')
  run('vevent/optg-fire', 'set nowrap')
  attempt('vevent/optionset-global', vim.api.nvim_eval, 'g:AUEV')
  dump('vevent/optionset-global/log')
  vim.cmd('silent! noautocmd set wrap')

  -- BufModifiedSet has no v:event and, measured, does not fire at all in
  -- `--headless -l` -- not from nvim_buf_set_lines, not from
  -- `:normal! i`, not after `:set nomodified`.  It stays in §18's watch
  -- list (an event that starts firing is as much a regression as one
  -- that stops), but it gets no case of its own here.

  -- nvim_exec_autocmds carries arbitrary `data` into the callback's
  -- argument table (not into v:event).
  reset()
  vim.g.AUDATA = vim.NIL
  vim.api.nvim_create_autocmd('User', {
    pattern = 'AuData',
    callback = function(args)
      vim.fn.Au('data')
      _G.AU_ARGS = args
    end,
  })
  attempt('vevent/data-exec', vim.api.nvim_exec_autocmds, 'User', {
    pattern = 'AuData',
    data = { a = 1, b = { 'x', 'y' }, c = true },
  })
  dump('vevent/data-log')
  local args = _G.AU_ARGS or {}
  -- `id` is a monotonic autocmd id; it is deterministic for a fixed
  -- section order but says nothing, so only its presence is recorded.
  args.id = args.id ~= nil and '<id>' or nil
  answer('vevent/data-args', args)
end)

-- ---------------------------------------------------------------------
-- §15  The API surface
-- ---------------------------------------------------------------------
section('api', function()
  reset()
  local id1 = vim.api.nvim_create_autocmd('User', {
    pattern = 'AuApi',
    command = 'call Au("cmd")',
    desc = 'a command handler',
  })
  local id2 = vim.api.nvim_create_autocmd('User', {
    pattern = { 'AuApi', 'AuOther' },
    callback = function()
      vim.fn.Au('cb')
    end,
    desc = 'a lua handler',
  })
  -- ids are a monotonic counter shared with every other section, so
  -- their difference is the answer, not their value.  Note that one
  -- create with two patterns is one id, not two.
  answer('api/created', { first = idof(id1), delta = id2 - id1 })
  attempt('api/exec', vim.api.nvim_exec_autocmds, 'User', { pattern = 'AuApi' })
  dump('api/log')

  local function scrubbed(list)
    for _, entry in ipairs(list or {}) do
      entry.id = entry.id and '<id>' or nil
      entry.callback = entry.callback and '<function>' or nil
      entry.group = entry.group and '<group>' or nil
    end
    return list
  end
  local function getau(label, opts)
    local ok, res = pcall(vim.api.nvim_get_autocmds, opts)
    if not ok then
      emit(label, '!', esc(errtext(res)))
      struct(label, { err = errtext(res) })
      return
    end
    answer(label, scrubbed(res))
  end
  answer('api/get-all', scrubbed(vim.api.nvim_get_autocmds({ event = 'User' })))
  answer(
    'api/get-pat',
    scrubbed(vim.api.nvim_get_autocmds({ event = 'User', pattern = 'AuOther' }))
  )
  getau('api/get-noargs', {})
  getau('api/get-badevent', { event = 'NoSuchEvent' })
  getau('api/get-badgroup', { group = 'NoSuchGroup' })

  -- A callback that returns true deletes itself, which is ++once by
  -- another road and goes through the same deferred-deletion path.
  reset()
  answer('api/selfdel', idof(vim.api.nvim_create_autocmd('User', {
    pattern = 'AuSelf',
    callback = function()
      vim.fn.Au('self')
      return true
    end,
  })))
  answer('api/selfdel-other', idof(vim.api.nvim_create_autocmd('User', {
    pattern = 'AuSelf',
    command = 'call Au("after")',
  })))
  attempt('api/selfdel-f1', vim.api.nvim_exec_autocmds, 'User', { pattern = 'AuSelf' })
  dump('api/selfdel-log1')
  attempt('api/selfdel-f2', vim.api.nvim_exec_autocmds, 'User', { pattern = 'AuSelf' })
  dump('api/selfdel-log2')

  -- `once` and `nested` through the API rather than through `++once` /
  -- `++nested` on the command line: a different code path into the same
  -- two AutoCmd flags.
  reset()
  answer('api/once', idof(vim.api.nvim_create_autocmd('User', {
    pattern = 'AuApiOnce',
    command = 'call Au("api-once")',
    once = true,
  })))
  answer('api/once-plain', idof(vim.api.nvim_create_autocmd('User', {
    pattern = 'AuApiOnce',
    command = 'call Au("api-plain")',
  })))
  attempt('api/once-f1', vim.api.nvim_exec_autocmds, 'User', { pattern = 'AuApiOnce' })
  dump('api/once-log1')
  attempt('api/once-f2', vim.api.nvim_exec_autocmds, 'User', { pattern = 'AuApiOnce' })
  dump('api/once-log2')

  reset()
  answer('api/nested', idof(vim.api.nvim_create_autocmd('User', {
    pattern = 'AuApiNest',
    command = 'call Au("api-outer") | silent! edit two.txt',
    nested = true,
  })))
  answer('api/nested-inner', idof(vim.api.nvim_create_autocmd('BufReadPre', {
    pattern = 'two.txt',
    command = 'call Au("api-inner")',
  })))
  attempt('api/nested-f', vim.api.nvim_exec_autocmds, 'User', { pattern = 'AuApiNest' })
  dump('api/nested-log')

  -- ... and the same shape with `nested` left off, which is what says
  -- the flag is being read rather than assumed.  Without this half a
  -- mutation that forces `nested` on for every api-created autocmd is
  -- indistinguishable from the truth (measured).
  reset()
  answer('api/nested-off', idof(vim.api.nvim_create_autocmd('User', {
    pattern = 'AuApiPlain',
    command = 'call Au("api-outer-plain") | silent! edit three.log',
  })))
  answer('api/nested-off-inner', idof(vim.api.nvim_create_autocmd('BufReadPre', {
    pattern = 'three.log',
    command = 'call Au("api-inner-plain")',
  })))
  attempt('api/nested-off-f', vim.api.nvim_exec_autocmds, 'User', { pattern = 'AuApiPlain' })
  dump('api/nested-off-log')

  -- Deleting and clearing.
  reset()
  local ids = {}
  for i = 1, 3 do
    ids[i] = vim.api.nvim_create_autocmd('User', {
      pattern = 'AuDel',
      command = ('call Au("d%d")'):format(i),
    })
  end
  attempt('api/del', vim.api.nvim_del_autocmd, ids[2])
  attempt('api/del-again', vim.api.nvim_del_autocmd, ids[2])
  attempt('api/del-bogus', vim.api.nvim_del_autocmd, 999999)
  attempt('api/exec-del', vim.api.nvim_exec_autocmds, 'User', { pattern = 'AuDel' })
  dump('api/del-log')
  attempt('api/clear', vim.api.nvim_clear_autocmds, { event = 'User', pattern = 'AuDel' })
  attempt('api/exec-cleared', vim.api.nvim_exec_autocmds, 'User', { pattern = 'AuDel' })
  dump('api/clear-log')

  -- The validation arms: every one of these is a distinct error text.
  reset()
  attempt('api/err-both', vim.api.nvim_create_autocmd, 'User', {
    pattern = 'X',
    command = 'echo 1',
    callback = 'Foo',
  })
  attempt('api/err-neither', vim.api.nvim_create_autocmd, 'User', { pattern = 'X' })
  attempt('api/err-event', vim.api.nvim_create_autocmd, 'NoSuchEvent', { pattern = 'X' })
  attempt('api/err-eventtype', vim.api.nvim_create_autocmd, 42, { pattern = 'X' })
  attempt('api/err-patbuf', vim.api.nvim_create_autocmd, 'User', {
    pattern = 'X',
    buffer = 0,
    command = 'echo 1',
  })
  attempt('api/err-badbuf', vim.api.nvim_create_autocmd, 'BufEnter', {
    buffer = 999999,
    command = 'echo 1',
  })
  attempt('api/err-group', vim.api.nvim_create_autocmd, 'User', {
    pattern = 'X',
    group = 'NoSuchGroup',
    command = 'echo 1',
  })
  attempt('api/err-groupid', vim.api.nvim_create_autocmd, 'User', {
    pattern = 'X',
    group = 999999,
    command = 'echo 1',
  })
  attempt('api/err-cbtype', vim.api.nvim_create_autocmd, 'User', {
    pattern = 'X',
    callback = 42,
  })
  attempt('api/err-clear-nothing', vim.api.nvim_clear_autocmds, {})
  attempt('api/err-clear-both', vim.api.nvim_clear_autocmds, {
    pattern = 'X',
    buffer = 0,
  })
  attempt('api/err-exec-badevent', vim.api.nvim_exec_autocmds, 'NoSuchEvent', {})
  attempt('api/err-augroup-badtype', vim.api.nvim_create_augroup, 42, {})
  answer('api/augroup-empty', idof(vim.api.nvim_create_augroup('', {})))
end)

-- ---------------------------------------------------------------------
-- §16  The :autocmd listing
-- ---------------------------------------------------------------------
section('listing', function()
  reset()
  run('listing/a', 'autocmd User AuL1 call Au("1")')
  mkgroup('GL')
  run('listing/b', 'autocmd GL User AuL2 call Au("2")')
  run('listing/c', 'autocmd BufNewFile *.txt,*.c call Au("3")')
  run('listing/open', 'noautocmd edit one.txt')
  run('listing/d', ('autocmd BufReadPre <buffer=%d> call Au("4")'):format(vim.fn.bufnr('%')))
  run('listing/e', 'autocmd GL BufWritePost * ++once call Au("5")')
  vim.api.nvim_create_autocmd('User', {
    pattern = 'AuL6',
    callback = function() end,
    desc = 'a described lua handler',
  })

  local function listing(label, cmd)
    local ok, out = pcall(vim.api.nvim_exec2, cmd, { output = true })
    if not ok then
      emit(label, '!', esc(errtext(out)))
      return
    end
    local n = 0
    for line in (out.output or ''):gmatch('[^\n]+') do
      n = n + 1
      emit(label, string.format('%02d', n), esc(scrub(line)))
    end
    if n == 0 then
      emit(label, '--', '(empty)')
    end
  end

  listing('listing/all', 'autocmd')
  listing('listing/event', 'autocmd User')
  listing('listing/group', 'autocmd GL')
  listing('listing/groupevent', 'autocmd GL BufWritePost')
  listing('listing/pat', 'autocmd BufNewFile *.txt')
  listing('listing/nomatch', 'autocmd User NoSuchPattern')
  listing('listing/star', 'autocmd * *.txt')
  listing('listing/badevent', 'autocmd NoSuchEvent')
end)

-- ---------------------------------------------------------------------
-- §17  exists('#...')
-- ---------------------------------------------------------------------
section('exists', function()
  reset()
  run('exists/a', 'autocmd User AuE1 call Au("1")')
  mkgroup('GX')
  run('exists/b', 'autocmd GX BufNewFile *.txt call Au("2")')
  run('exists/c', 'edit one.txt')
  run('exists/d', 'autocmd BufWritePre <buffer> call Au("3")')
  local QUERIES = {
    '#User',
    '#User#AuE1',
    '#User#AuNope',
    '#BufNewFile',
    '#BufNewFile#*.txt',
    '#BufNewFile#*.c',
    '#GX',
    '#GX#BufNewFile',
    '#GX#BufNewFile#*.txt',
    '#GX#User',
    '#NoSuchGroup',
    '#NoSuchGroup#User',
    '#NoSuchEvent',
    '#BufWritePre#<buffer>',
    '#BufWritePre#<buffer=1>',
    '#',
    '##',
    '#User#',
  }
  local out = {}
  for _, q in ipairs(QUERIES) do
    local ok, res = pcall(vim.fn.exists, q)
    out[q] = ok and res or errtext(res)
    emit('exists/q', esc(q), '=', esc(tostring(out[q])))
  end
  struct('exists/q', out)
end)

-- ---------------------------------------------------------------------
-- §18  Real buffer / window / tab events, in order
-- ---------------------------------------------------------------------
section('bufwin', function()
  -- One autocmd per event, with the event's own name passed in:
  -- `expand('<event>')` answers empty (measured), so a single
  -- comma-separated definition could not say which event fired -- and
  -- *which* is the whole artifact here.
  local WATCH = {
    'BufAdd', 'BufNew', 'BufDelete', 'BufWipeout', 'BufEnter', 'BufLeave',
    'BufWinEnter', 'BufWinLeave', 'BufNewFile', 'BufReadPre', 'BufReadPost',
    'BufFilePre', 'BufFilePost', 'BufHidden', 'BufUnload', 'BufWrite',
    'BufWritePre', 'BufWritePost', 'FileType', 'FileReadPre', 'FileReadPost',
    'FileWritePre', 'FileWritePost', 'WinNew', 'WinEnter', 'WinLeave',
    'WinClosed', 'WinScrolled', 'WinResized', 'TabNew', 'TabEnter',
    'TabLeave', 'TabClosed', 'InsertEnter', 'InsertLeave', 'ModeChanged',
    'TextChanged', 'CursorMoved', 'BufModifiedSet',
  }

  local function watch()
    for _, ev in ipairs(WATCH) do
      local err = command(('autocmd %s * call AuEv("%s")'):format(ev, ev))
      if err then
        emit('bufwin/watch', ev, '!', esc(err))
      end
    end
  end

  local function scenario(label, cmds)
    reset()
    run(label .. '/open', 'noautocmd edit one.txt')
    watch()
    for _, cmd in ipairs(cmds) do
      run(label .. '/cmd', cmd)
    end
    dump(label)
  end

  scenario('bufwin/edit', { 'edit two.txt' })
  scenario('bufwin/split', { 'split' })
  scenario('bufwin/splitfile', { 'split two.txt' })
  scenario('bufwin/close', { 'split', 'close' })
  scenario('bufwin/newbuf', { 'enew' })
  scenario('bufwin/badd', { 'badd three.log' })
  scenario('bufwin/bdelete', { 'badd three.log', 'bdelete three.log' })
  scenario('bufwin/bwipe', { 'badd three.log', 'bwipeout three.log' })
  scenario('bufwin/tabnew', { 'tabnew' })
  scenario('bufwin/tabclose', { 'tabnew', 'tabclose' })
  scenario('bufwin/write', { 'write! ' .. work .. '/tree/written.txt' })
  scenario('bufwin/filetype', { 'setfiletype lua' })
  scenario('bufwin/rename', { 'file renamed.txt' })
  scenario('bufwin/insert', { 'normal! ihi\27' })
  scenario('bufwin/hide', { 'set hidden', 'edit two.txt', 'set nohidden' })
  scenario('bufwin/onlywin', { 'split two.txt', 'only' })
end)

-- ---------------------------------------------------------------------
-- §19  Malformed :autocmd
-- ---------------------------------------------------------------------
section('errors', function()
  reset()
  local BAD = {
    'autocmd NoSuchEvent * echo 1',
    'autocmd User',
    'autocmd! User AuE nested',
    'autocmd User AuE ++nope call Au("x")',
    'autocmd User AuE ++once ++once call Au("x")',
    'autocmd <buffer=nope> User echo 1',
    'autocmd BufEnter <buffer=99999> echo 1',
    'augroup',
    'augroup!',
    'augroup! *',
    'doautocmd',
    'doautocmd User NoSuchPattern extra args',
    'doautoall NoSuchEvent',
    'autocmd * * echo 1',
    'autocmd! * *.txt',
    'autocmd User AuE ++nested nested call Au("x")',
  }
  for i, cmd in ipairs(BAD) do
    local err = command(cmd)
    emit('errors', string.format('%02d', i), esc(cmd), '->', esc(err or '(ok)'))
    struct('errors/' .. string.format('%02d', i), { cmd = cmd, err = err })
  end
  -- `:autocmd` with a pattern that is only a group name, and the
  -- "group or event?" ambiguity a group named like an event creates.
  reset()
  run('errors/amb1', 'autocmd! BufEnter')
  run('errors/amb2', 'autocmd BufEnter * call Au("event-not-group")')
  run('errors/amb3', 'doautocmd BufEnter one.txt')
  dump('errors/amblog')
end)

-- ---------------------------------------------------------------------
-- Run.
-- ---------------------------------------------------------------------
for _, sec in ipairs(SECTIONS) do
  if not only or sec.name:match(only) then
    emit('==', sec.name)
    local ok, err = pcall(sec.fn)
    if not ok then
      emit('!!', sec.name, esc(errtext(err)))
      struct('!!' .. sec.name, { err = errtext(err) })
    end
  end
end
structfd:close()
