-- Driver for the Ex-command differential sweep; see cmdsweep.sh.
--
-- Covers the two API entry points into the Ex layer -- nvim_parse_cmd()
-- and nvim_cmd() -- and through them the B17 family: ex_docmd's parser
-- (lookup/address/modifier/argopt/api), ex_cmds.rs, ex_cmds2.rs,
-- ex_eval.rs, ex_session.rs, usercmd.rs, debugger.rs, help.rs,
-- digraph.rs, cmdhist.rs and arglist.  Before this, the *whole* of that
-- surface had only the functional suite behind it.
--
-- Every case is an Ex command line.  It goes through nvim_parse_cmd and
-- the resulting Dict is reported in full; where the line is safe to run,
-- the Dict goes on into nvim_cmd and the answer is what changed:
-- captured output, the raised error, the scratch buffer, the cursor, the
-- b:changedtick delta, the window/tab/buffer shape, and whatever extra
-- question the case asked.
--
--   s01 every command name the binary knows (getcompletion 'command'),
--       plain and with a bang -- the cmdnames table as a golden.
--   s02 every 1..4-character prefix of every command name: which
--       command an abbreviation resolves to, and which are ambiguous.
--       find_ex_command's whole table walk.
--   s03 ranges: %, ., $, N, marks, /pat/, ?pat?, \/ \? \&, +N, -N, ;,
--       0, out-of-range, reversed, and the addr_type each is counted in.
--   s04 counts and registers: the EX_COUNT and EX_REGSTR arms.
--   s05 command modifiers: every one alone, abbreviated, stacked, with
--       counts (3tab, verbose 5, filter /pat/), and modifier-only lines.
--   s06 arguments: quoting, backslash escapes, bars, trailing comments,
--       EX_TRLBAR vs EX_NOTRLCOM, EX_NOSPC, the :map-family splitter.
--   s07 file arguments and expandables: % # ## <cword> <cWORD> <cfile>
--       <afile> <abuf>, the :p:h modifiers, and magic.file on and off.
--   s08 +cmd and ++opt (ex_docmd/argopt.rs).
--   s09 user commands: the -nargs/-range/-count/-addr/-complete/-bang/
--       -bar/-register/-buffer matrix, defined, listed, parsed, invoked
--       and deleted; every <...> substitution in a replacement string.
--   s10 :substitute: separators, every flag, offsets, \=, :smagic,
--       :snomagic, the repeat forms (:s, :&, :&&, :~).
--   s11 :global / :vglobal, plain and nested.
--   s12 control flow, parse side: :if :elseif :else :endif :while :for
--       :try :catch :finally :endtry :throw :function :return, plus a
--       few executed through one nvim_exec2 body.
--   s13 the ex_cmds.rs shell executed: :move :copy :t :delete :yank
--       :put :join :> :< :normal :sort :print :number :list :z :read.
--   s14 nvim_cmd over hand-built Dicts: every key wrong, missing,
--       out of range or of the wrong type -- the API validation surface,
--       which no parse output can reach.
--   s15 round trip: parse a line, hand the Dict straight back to
--       nvim_cmd, and record that the two agree.
--   s16 deliberately malformed input: the error-message artifact.
--   s17 the rest of B17 through its commands: :help, :digraph(s),
--       :history, :args/:argadd/:argdelete/:argdo, :mksession (golden
--       file), :breakadd/:breaklist/:breakdel, :scriptnames, :source,
--       :runtime.
--   s20 a block of commands run *uncaptured* with 'report' at 0, so the
--       .stderr artifact carries the real message path -- "N more
--       lines", "N substitutions on N lines", the listings, the
--       E-numbers.
--   s91 CRASHPROBE: the inputs that may kill the editor, one child each,
--       so a crash is one diffable ABORTED row rather than a truncated
--       report.
--
-- Everything printed has to be reproducible across two builds run
-- minutes apart and from two working directories, so the report carries
-- no address, pid, wall-clock time, buffer handle or path outside the
-- work directory.  b:changedtick and the buffer/window/tab handles are
-- monotonic counters; only *deltas* and *counts* are recorded.
--
-- CMDSWEEP_ONLY is a Lua pattern matched against each section name; it
-- exists for iterating on one section, not for gating.
-- CMDSWEEP_TRACE=1 mirrors each section name to stderr, which is the
-- only way to see where a wedged run stopped, and must be off for a
-- baseline because it writes into the .stderr artifact.

local work = assert(os.getenv('CMD_WORK'), 'CMD_WORK unset')

-- Child mode.  s91 runs its cases in a child nvim, because an input that
-- aborts the editor would otherwise take the whole report with it.  The
-- child re-runs this same file and prints from index argv[2] onward,
-- unbuffered, so the parent can name the case it died on and resume.
local argv = _G.arg or {}
local child_from = (argv[1] == '--child') and tonumber(argv[2]) or nil

local structfd
if not child_from then
  structfd = assert(io.open(assert(os.getenv('CMD_STRUCT'), 'CMD_STRUCT unset'), 'w'))
end
local only = os.getenv('CMDSWEEP_ONLY')
if only == '' then
  only = nil
end
local trace = os.getenv('CMDSWEEP_TRACE') == '1'

io.stdout:setvbuf(child_from and 'no' or 'line')

local function emit(...)
  io.write(table.concat({ ... }, ' '), '\n')
end

local runtime = os.getenv('VIMRUNTIME') or ''
local script = debug.getinfo(1, 'S').source:sub(2)

--- Strip the bits of an answer that name where -- or when -- the run
--- happened.  Sorting happens after this, never before.
local function scrub(text)
  text = tostring(text)
  text = text:gsub(vim.pesc(script), '<SCRIPT>'):gsub('%.%.%.[^%s\'"]-cmdsweep%.lua', '<SCRIPT>')  -- LuaJIT elides a chunk name past ~60 chars
  -- A Lua traceback names the line THIS FILE's frames sit on, so any
  -- edit above a k91 case that raises one re-baselines its row -- the
  -- path was scrubbed and the line number was not (B16-5's traceback
  -- trap, in the half that was missed).
  text = text:gsub('<SCRIPT>:%d+', '<SCRIPT>:N')
  -- ... and `:command`'s "Last set from <file> line N" names the line
  -- the sweep defined the user command on, which is the same hazard
  -- wearing different punctuation: 45 c09l rows moved when this file
  -- grew by nine lines ABOVE the definition.
  text = text:gsub('(<SCRIPT> line )%d+', '%1N')
  text = text:gsub(vim.pesc(work), '<WORK>')
  text = text:gsub(vim.pesc(work:sub(2)), '<WORK>')
  if runtime ~= '' then
    text = text:gsub(vim.pesc(runtime), '<RUNTIME>')
  end
  text = text:gsub('0x%x+', '<ADDR>')
  text = text:gsub('<lambda>%d+', '<lambda>')
  text = text:gsub('<SNR>%d+_', '<SNR>_')
  text = text:gsub('%S*/target/debug/nvim', '<NVIM>')
  text = text:gsub('%S*/nvim%-%x+', '<NVIM>')
  -- The Lua core modules are named by a *relative* chunk name that
  -- `:scriptnames` and `:command`'s "Last set from" resolve against the
  -- cwd nvim was exec'd in -- not $VIMRUNTIME, and not $WORK.  Without
  -- this the artifact is a function of where the verify script ran.
  text = text:gsub('%S*/vim/_core/', '<CORE>/')
  -- v:servername is a fresh socket path per process.
  text = text:gsub('/tmp/nvim%.[%w_]+/[%w]+/nvim%.%d+%.%d+', '<SERVER>')
  -- `:undo`'s own report is "N changes; before #M  T seconds ago".  The
  -- age is a wall clock and was the *only* row that differed between two
  -- runs of the same binary; the change count and the sequence number
  -- are one counter for the whole process, so they are a function of
  -- every case above the one that printed them.  opsweep scrubs the same
  -- text in its shell driver, over stderr -- here the message arrives
  -- through nvim_cmd's own capture, which only this function sees.
  for _, unit in ipairs({ 'second', 'minute', 'hour', 'day' }) do
    text = text:gsub('%d+ ' .. unit .. 's? ago', 'N ago')
  end
  text = text:gsub('%d+ changes?;', 'N changes;')
  text = text:gsub('before #%d+', 'before #N')
  text = text:gsub('after #%d+', 'after #N')
  return text
end

--- Escape to one printable line: a byte difference has to show in the
--- diff, and a report line has to stay a report line.
local function esc(bytes)
  return (tostring(bytes):gsub('[^\32-\126]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

--- A case label is the *input itself*, percent-escaped.  Injective by
--- construction -- which a punctuation-to-token map is not, and this
--- corpus is nothing but punctuation.  The DUPLICATE-LABEL check below
--- is the standing proof that it stayed injective.
local function tag(text)
  return (tostring(text):gsub('[^%w]', function(c)
    return string.format('%%%02x', c:byte())
  end))
end

-- ---------------------------------------------------------------------
-- Canonical dump.  Verbatim from opsweep.lua / varsweep.lua: the
-- artifacts are read side by side often enough that they must escape and
-- sort the same way.
-- ---------------------------------------------------------------------
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
  if structfd then
    structfd:write(label, ' ', canon(value), '\n')
  end
end

--- Normalise an error to its message.  A pcall against vim.api prefixes
--- the Lua source position, which is a line number in *this* file and
--- would re-baseline the whole artifact on any edit above it.
local function errtext(res)
  local text = scrub(tostring(res))
  text = text:gsub('^.-:%d+: ', '')
  text = text:gsub('^nvim_exec2%(%), line %d+: ', '')
  text = text:gsub('^Vim:', '')
  text = text:gsub('^Vim%b():', '')
  return text
end

-- ---------------------------------------------------------------------
-- The world a case runs in.
-- ---------------------------------------------------------------------

--- All the options any section touches, at a fixed value, so a case that
--- forgets to restore one cannot poison its neighbours.
local DEFAULTS = table.concat({
  'set magic gdefault& ignorecase& smartcase& wrapscan incsearch&',
  'set report=9999 shortmess=filnxtToOFS nomore noshowmode belloff=all',
  'set noswapfile nobackup nowritebackup hidden autochdir&',
  'set shiftwidth=8 tabstop=8 softtabstop=0 noexpandtab noshiftround',
  'set textwidth=0 virtualedit= selection=inclusive nostartofline',
  'set autoindent& smartindent& cindent& lisp& formatoptions=tcq',
  'set wildignore= suffixes& isfname&vim path=. cdpath=,, wildmenu&',
  'set sessionoptions=buffers,curdir,folds,help,tabpages,winsize',
  -- NOT `verbose=0`.  In `nvim -l` batch mode 'verbose' starts at 1, and
  -- that is what routes nvim's own messages to stderr; setting it to
  -- *zero* silences every message in the process -- not only the verbose
  -- ones -- and the .stderr artifact goes empty while every other
  -- artifact looks healthy.  s20 exists to catch exactly that, so the
  -- sweep pins 'verbose' at the value the mode started with.
  'set viewoptions=folds,cursor,curdir eventignore= verbose=1',
  -- 'inccommand' is GLOBAL: without pinning it here, s10's c10i cases
  -- leak their setting into every later section, and `:mkvimrc` in s17
  -- records it.
  'set inccommand&',
  'set cpoptions=aABceFs_ maxfuncdepth=100 debug= foldenable& foldmethod=manual',
}, ' | ')

-- Every register the sweep clears between cases.  `_` is a black hole by
-- definition; the special ones (. : % # / =) are read-only, seeded, or
-- their own question.
local REGS = { '"', '-', '+', '*' }
for c = 0, 9 do
  REGS[#REGS + 1] = tostring(c)
end
for c = string.byte('a'), string.byte('z') do
  REGS[#REGS + 1] = string.char(c)
end

local BUF

local function quiet(src)
  local ok, res = pcall(vim.api.nvim_exec2, src, { output = false })
  if not ok then
    return errtext(res)
  end
  return nil
end

--- The scratch buffer's fixture: twenty short, distinguishable lines, so
--- a range answer names which lines moved rather than "some lines".
local FIXTURE = {}
for i = 1, 20 do
  FIXTURE[i] = string.format('L%02d alpha beta', i)
end
FIXTURE[5] = 'L05 MATCH beta'
FIXTURE[6] = ''
FIXTURE[7] = 'L07 MATCH gamma'
FIXTURE[12] = '\tL12 indented'
FIXTURE[15] = 'L15 caf\xc3\xa9 wide'

local base_tick = 0

--- Wipe everything a case might have created and put the world back:
--- one tab, one window, one (scratch) buffer, an empty arglist, no
--- extra marks, the fixture, the defaults.
local function reset(lines, pos, opts)
  pcall(vim.api.nvim_exec2, 'silent! tabonly!', { output = false })
  pcall(vim.api.nvim_exec2, 'silent! only!', { output = false })
  pcall(vim.api.nvim_set_current_buf, BUF)
  for _, b in ipairs(vim.api.nvim_list_bufs()) do
    if b ~= BUF then
      pcall(vim.api.nvim_buf_delete, b, { force = true })
    end
  end
  quiet('silent! %argdelete')
  -- `:file name` renames the scratch buffer for good, and s13 runs three
  -- of them.  Without this the buffer *name* -- which every S line
  -- reports -- is a function of the last case that renamed it, several
  -- sections above.
  pcall(vim.api.nvim_buf_set_name, BUF, '')
  quiet(DEFAULTS)
  quiet('silent! normal! zE')
  -- B17-5.  Two counters were global and MONOTONIC across the whole run,
  -- so `:undo`'s tick delta (s13) and `:history`'s entry numbers (s17)
  -- were functions of how many cases had run BEFORE them -- adding a case
  -- anywhere above silently re-baselined eight rows in sections it had
  -- nothing to do with.  The undo history is cleared with the documented
  -- `:help clear-undo` idiom (a change made while 'undolevels' is -1
  -- replaces the whole tree), and the five histories with `histdel()`.
  quiet('let g:cs_ul = &l:undolevels | setlocal undolevels=-1')
  vim.api.nvim_buf_set_lines(BUF, 0, -1, false, lines or FIXTURE)
  quiet('let &l:undolevels = g:cs_ul | unlet! g:cs_ul')
  quiet(
    'call histdel(":") | call histdel("/") | call histdel("=") '
      .. '| call histdel("@") | call histdel(">")'
  )
  for _, m in ipairs({ '<', '>', '[', ']', 'a', 'b', 'c' }) do
    pcall(vim.api.nvim_buf_del_mark, BUF, m)
  end
  -- Two named marks every range case can use, set from here rather than
  -- from a `:normal` so the case's own keys are the only ones in play.
  pcall(vim.api.nvim_buf_set_mark, BUF, 'a', 3, 0, {})
  pcall(vim.api.nvim_buf_set_mark, BUF, 'b', 9, 0, {})
  -- Registers are global, so `:registers`, `:put` and `:delete x` would
  -- otherwise answer about every case above this one.  Cleared, not
  -- deleted, and then seeded with the two the corpus names.
  for _, r in ipairs(REGS) do
    pcall(vim.fn.setreg, r, {})
  end
  pcall(vim.fn.setreg, '/', '')
  pcall(vim.fn.setreg, 'r', 'REG')
  pcall(vim.fn.setreg, 'a', 'AAA')
  pcall(vim.api.nvim_win_set_cursor, 0, pos or { 1, 0 })
  if opts and opts ~= '' then
    quiet('set ' .. opts)
  end
  base_tick = vim.api.nvim_buf_get_var(BUF, 'changedtick')
end

-- ---------------------------------------------------------------------
-- Reporting.
-- ---------------------------------------------------------------------

local SEEN = {}

local function label_once(label)
  -- A collision is invisible in a 30,000-line report and turns two cases
  -- into one; say so in the artifact rather than in a comment.
  if SEEN[label] then
    emit(label, 'DUPLICATE-LABEL', tostring(SEEN[label] + 1))
  end
  SEEN[label] = (SEEN[label] or 0) + 1
end

-- The mods sub-dictionary at rest.  Only what differs from this is
-- printed in the .txt line; .struct keeps the whole thing, always.
local MOD_DEFAULT = {
  browse = false,
  confirm = false,
  emsg_silent = false,
  hide = false,
  horizontal = false,
  keepalt = false,
  keepjumps = false,
  keepmarks = false,
  keeppatterns = false,
  lockmarks = false,
  noautocmd = false,
  noswapfile = false,
  sandbox = false,
  silent = false,
  split = '',
  tab = -1,
  unsilent = false,
  verbose = -1,
  vertical = false,
}

--- The parse Dict as one line: every key that carries information, and
--- of `mods` only what is not at rest.  The full shape is in .struct.
local function flat(d)
  local parts = {}
  parts[#parts + 1] = 'cmd=' .. esc(tostring(d.cmd))
  if d.bang then
    parts[#parts + 1] = 'bang'
  end
  if d.range then
    local r = {}
    for _, n in ipairs(d.range) do
      r[#r + 1] = tostring(n)
    end
    parts[#parts + 1] = 'range=[' .. table.concat(r, ',') .. ']'
  end
  if d.count ~= nil then
    parts[#parts + 1] = 'count=' .. tostring(d.count)
  end
  if d.reg ~= nil and d.reg ~= '' then
    parts[#parts + 1] = 'reg=' .. esc(d.reg)
  end
  local args = {}
  for _, a in ipairs(d.args or {}) do
    args[#args + 1] = esc(scrub(a))
  end
  parts[#parts + 1] = 'args=[' .. table.concat(args, '|') .. ']'
  parts[#parts + 1] = 'nargs=' .. tostring(d.nargs)
  parts[#parts + 1] = 'addr=' .. tostring(d.addr)
  parts[#parts + 1] = 'next=' .. esc(scrub(d.nextcmd or ''))
  local magic = d.magic or {}
  parts[#parts + 1] = 'magic=' .. (magic.file and 'f' or '-') .. (magic.bar and 'b' or '-')
  local mods = d.mods or {}
  local extra = {}
  local names = {}
  for k in pairs(mods) do
    names[#names + 1] = k
  end
  table.sort(names)
  for _, k in ipairs(names) do
    if k == 'filter' then
      local f = mods.filter or {}
      if (f.pattern or '') ~= '' or f.force then
        extra[#extra + 1] = 'filter=' .. esc(f.pattern or '') .. (f.force and '!' or '')
      end
    elseif mods[k] ~= MOD_DEFAULT[k] then
      extra[#extra + 1] = k .. '=' .. tostring(mods[k])
    end
  end
  parts[#parts + 1] = 'mods{' .. table.concat(extra, ',') .. '}'
  return table.concat(parts, ' ')
end

--- Parse one line and report it.  Returns the Dict, or nil when the
--- parse raised -- the error is an artifact row either way.
local function parse(label, line, opts)
  label_once(label)
  local ok, res = pcall(vim.api.nvim_parse_cmd, line, opts or {})
  if ok then
    emit(label, 'P', flat(res))
    struct(label, res)
    return res
  end
  emit(label, '!', esc(errtext(res)))
  struct(label, { err = errtext(res) })
  return nil
end

--- The observable world after a command ran.  Handles are monotonic, so
--- what is recorded is a *count* of buffers/windows/tabs and a *delta*
--- of b:changedtick; the current buffer is named, scrubbed, because a
--- command that switched buffers is exactly what several cases ask.
local function worldline()
  local view = vim.fn.winsaveview()
  local name = vim.api.nvim_buf_get_name(0)
  -- `expand('#')` raises E194 when there is no alternate file, and an
  -- emsg raised from the *reporter* aborts the whole section (nine of
  -- them, the first time this ran at a 'verbose' that let the error
  -- through).  bufname() answers the empty string instead.
  local okalt, alt = pcall(vim.fn.bufname, '#')
  alt = okalt and alt or ''
  local tick = 0
  if vim.api.nvim_buf_is_valid(BUF) then
    tick = vim.api.nvim_buf_get_var(BUF, 'changedtick') - base_tick
  end
  return string.format(
    'c=%d,%d t=%d bufs=%d wins=%d tabs=%d cur=%s alt=%s args=%d mod=%s',
    view.lnum,
    view.col,
    tick,
    #vim.api.nvim_list_bufs(),
    #vim.api.nvim_tabpage_list_wins(0),
    #vim.api.nvim_list_tabpages(),
    esc(scrub(name)),
    esc(scrub(alt)),
    vim.fn.argc(),
    tostring(vim.bo.modified)
  ),
    {
      c = { view.lnum, view.col },
      t = tick,
      bufs = #vim.api.nvim_list_bufs(),
      wins = #vim.api.nvim_tabpage_list_wins(0),
      tabs = #vim.api.nvim_list_tabpages(),
      cur = scrub(name),
      alt = scrub(alt),
      args = vim.fn.argc(),
      mod = vim.bo.modified,
    }
end

--- One executing case.  `o` may carry:
---   dict    a hand-built Dict, instead of parsing `line`
---   lines   the scratch fixture to start from
---   pos     the cursor to start from
---   opts    a `:set` argument string applied on top of DEFAULTS
---   pre     Vimscript run after the reset and before the command
---   after   a Vimscript expression asked once the command has run
---   copts   the opts Dict handed to nvim_cmd (default {output=true})
---   noparse do not report the parse Dict (s13/s17 read as execution)
local function exec(label, line, o)
  o = o or {}
  label_once(label)
  reset(o.lines, o.pos, o.opts)
  if o.pre then
    local err = quiet(o.pre)
    if err then
      emit(label, '!pre', esc(err))
    end
    base_tick = vim.api.nvim_buf_is_valid(BUF)
        and vim.api.nvim_buf_get_var(BUF, 'changedtick')
      or base_tick
  end
  local dict = o.dict
  if dict == nil then
    local ok, res = pcall(vim.api.nvim_parse_cmd, line, {})
    if not ok then
      emit(label, '!parse', esc(errtext(res)))
      struct(label, { parse_err = errtext(res) })
      return
    end
    dict = res
    if not o.noparse then
      emit(label, 'P', flat(res))
    end
    -- nvim_parse_cmd sets `reg` to the EMPTY STRING for every EX_REGSTR
    -- command that named no register (`:delete`, `:yank`, `:put`, ...),
    -- and nvim_cmd rejects an empty `reg` with "expected single
    -- character" -- so the documented round trip fails for that whole
    -- family, and a section that fed the Dict straight back would never
    -- execute one of them.  Dropped here so the *command* is what is
    -- under test; s15's `c15raw/` cases keep the empty key, and gate the
    -- bug itself.  (Upstream neovim does the same thing; filed as
    -- api-parse-cmd-empty-reg-roundtrip.)
    if not o.rawreg and dict.reg == '' then
      dict.reg = nil
    end
  end
  local ok, res
  if o.via == 'exec2' then
    -- The command *line*, not the Dict.  nvim_cmd's Dict is lossy in
    -- both directions (an EX_COUNT command's `range` and `count` both
    -- survive the parse, and executing them together means "count lines
    -- from range's end"), so `:1,3delete` through nvim_cmd is really
    -- `:3,5delete`.  Running the same corpus both ways makes the two
    -- entry points diffable against each other in one artifact.
    ok, res = pcall(vim.api.nvim_exec2, line, { output = true })
    if ok then
      res = res.output
    end
  else
    local copts = o.copts or { output = true }
    ok, res = pcall(vim.api.nvim_cmd, dict, copts)
  end
  local answer = {}
  if ok then
    emit(label, 'O', esc(scrub(res == nil and '<nil>' or tostring(res))))
    answer.out = res == nil and vim.NIL or res
  else
    emit(label, '!', esc(errtext(res)))
    answer.err = errtext(res)
  end
  local lines = vim.api.nvim_buf_is_valid(BUF)
      and vim.api.nvim_buf_get_lines(BUF, 0, -1, false)
    or { '<wiped>' }
  emit(label, 'B', esc(table.concat(lines, '\n')))
  local wl, wt = worldline()
  emit(label, 'S', wl)
  answer.b = lines
  answer.s = wt
  if o.after then
    local aok, ares = pcall(vim.fn.eval, 'string(' .. o.after .. ')')
    emit(label, 'A', aok and esc(scrub(ares)) or esc(errtext(ares)))
    answer.a = aok and scrub(ares) or { err = errtext(ares) }
  end
  struct(label, answer)
end

local SECTIONS = {}
local function section(name, fn)
  SECTIONS[#SECTIONS + 1] = { name = name, fn = fn }
end

-- ---------------------------------------------------------------------
-- s01 -- the cmdnames table as a golden.
-- ---------------------------------------------------------------------

-- Captured once, before anything defines or clears a user command, so
-- the enumeration is the binary's own table plus whatever the runtime
-- files installed -- and a change to either is a diff.
local ALLCMDS = {}

section('s01-names', function()
  emit('names', 'count', tostring(#ALLCMDS))
  for _, name in ipairs(ALLCMDS) do
    parse('c01/' .. tag(name), name)
    parse('c01!/' .. tag(name), name .. '!')
  end
end)

-- ---------------------------------------------------------------------
-- s02 -- abbreviation resolution.
-- ---------------------------------------------------------------------

section('s02-prefix', function()
  local seen, order = {}, {}
  for _, name in ipairs(ALLCMDS) do
    for n = 1, math.min(4, #name) do
      local p = name:sub(1, n)
      if not seen[p] then
        seen[p] = true
        order[#order + 1] = p
      end
    end
  end
  table.sort(order)
  emit('prefix', 'count', tostring(#order))
  for _, p in ipairs(order) do
    parse('c02/' .. tag(p), p)
  end
end)

-- ---------------------------------------------------------------------
-- s03 -- ranges.
-- ---------------------------------------------------------------------

-- Every range spelling, against a command that takes one.  Parsed only:
-- an address that names a search or a mark is resolved *at parse time*,
-- so the parse Dict already carries the whole answer, and executing
-- every one of them would be twenty deletes of the same fixture.
local RANGES = {
  '',
  '%',
  '.',
  '$',
  '1',
  '0',
  '7',
  '20',
  '21',
  '1,5',
  '5,1',
  '.,$',
  '.,.',
  '$,$',
  '1,$',
  '.+1',
  '.-1',
  '.+',
  '.-',
  '+3',
  '-2',
  '+',
  '-',
  '++',
  '--',
  "'a",
  "'b",
  "'a,'b",
  "'b,'a",
  "'z",
  "'a+2",
  "'<,'>",
  '/MATCH/',
  '/MATCH/,/gamma/',
  '?MATCH?',
  '/MATCH/+1',
  '/MATCH/;/gamma/',
  '\\/',
  '\\?',
  '\\&',
  '.;+2',
  '5;+2',
  '1;$',
  '.,+2',
  '3,-1',
  '1,2,3',
  '.,',
  ',',
  ',5',
  '9999',
  '.+9999',
  '$-9999',
}

section('s03-range', function()
  for _, r in ipairs(RANGES) do
    parse('c03d/' .. tag(r), r .. 'delete')
    parse('c03p/' .. tag(r), r .. 'print')
  end
  -- The same spellings against every addr_type, so that the address
  -- *counter* is exercised and not only the line one.  One command per
  -- type, from the table's own addr strings.
  local BYADDR = {
    line = 'print',
    arg = 'argdelete',
    buf = 'bdelete',
    load = 'bunload',
    win = 'wincmd',
    tab = 'tabclose',
    qf = 'cc',
    none = 'echo',
  }
  local names = {}
  for k in pairs(BYADDR) do
    names[#names + 1] = k
  end
  table.sort(names)
  for _, kind in ipairs(names) do
    for _, r in ipairs({ '', '%', '.', '$', '1', '1,5', '+2', "'a", '0' }) do
      parse('c03a/' .. kind .. '/' .. tag(r), r .. BYADDR[kind])
    end
  end
  -- A range with a bar and a following command: line1/line2 belong to
  -- the first command and `nextcmd` to the rest.
  for _, line in ipairs({
    '1,3d | 4,6d',
    '%s/a/b/ | echo "x"',
    '1,2print|2,3print',
    'g/x/d | echo 1',
    '1,3normal xx | echo 2',
  }) do
    parse('c03n/' .. tag(line), line)
  end
end)

-- ---------------------------------------------------------------------
-- s04 -- counts and registers.
-- ---------------------------------------------------------------------

section('s04-count', function()
  for _, line in ipairs({
    'buffer 1',
    'buffer 3',
    'buffer',
    '3buffer',
    'bdelete 1',
    'Next 2',
    'next 2',
    'argument 2',
    '2argument',
    'sleep 1',
    'sleep 100m',
    'z 5',
    'z=5',
    'z#5',
    'z-',
    'z+',
    'z^',
    'z.',
    'tabnext 2',
    '2tabnext',
    'tabnext +1',
    'wincmd 3 w',
    '3wincmd w',
    'cc 4',
    'copen 5',
    'copen',
    'resize 5',
    'resize +2',
    'delete a',
    'delete a 5',
    'yank b 3',
    '1,3delete * 5',
    'put +',
    'put',
    'put!',
    'put =[1,2]',
    'registers ab',
    'normal! 3x',
    'undo 5',
    'redo',
  }) do
    parse('c04/' .. tag(line), line)
  end
end)

-- ---------------------------------------------------------------------
-- s05 -- command modifiers.
-- ---------------------------------------------------------------------

local MODS = {
  'aboveleft',
  'abo',
  'belowright',
  'bel',
  'botright',
  'bo',
  'browse',
  'bro',
  'confirm',
  'conf',
  'hide',
  'hid',
  'horizontal',
  'hor',
  'keepalt',
  'keepa',
  'keepjumps',
  'keepj',
  'keepmarks',
  'kee',
  'keeppatterns',
  'keepp',
  'leftabove',
  'lefta',
  'lockmarks',
  'loc',
  'noautocmd',
  'noa',
  'noswapfile',
  'nos',
  'rightbelow',
  'rightb',
  'sandbox',
  'san',
  'silent',
  'sil',
  'silent!',
  'sil!',
  'tab',
  'topleft',
  'to',
  'unsilent',
  'uns',
  'verbose',
  'verb',
  'vertical',
  'vert',
}

section('s05-mods', function()
  for _, m in ipairs(MODS) do
    parse('c05/' .. tag(m), m .. ' echo "x"')
    parse('c05b/' .. tag(m), m)
  end
  for _, line in ipairs({
    '3tab split',
    '0tab split',
    '999tab split',
    'tab split',
    '5verbose set',
    '0verbose set',
    'verbose 5 set',
    'verbose set',
    'silent! silent echo 1',
    'unsilent silent echo 1',
    'vertical belowright split',
    'aboveleft vertical split',
    'topleft botright split',
    'botright topleft split',
    'leftabove rightbelow split',
    'keepalt keepjumps keepmarks keeppatterns lockmarks echo 1',
    'noautocmd noswapfile sandbox silent echo 1',
    'browse confirm hide echo 1',
    'filter /x/ ls',
    'filter! /x/ ls',
    'filter /a\\|b/ ls',
    'filter x ls',
    'filter ls',
    'filter /x/ filter /y/ ls',
    '2filter /x/ ls',
    'silent filter /x/ ls',
    'vertical 30 split',
    'tab 3 split',
    'verbose verbose echo 1',
    'silent 3tab vertical belowright split',
    -- A modifier abbreviation that is also a command prefix.
    'k',
    'ke',
    'kee',
    'keep',
    'vi',
    'vim',
  }) do
    parse('c05x/' .. tag(line), line)
  end
end)

-- ---------------------------------------------------------------------
-- s06 -- arguments: quoting, escaping, bars, comments.
-- ---------------------------------------------------------------------

section('s06-args', function()
  for _, line in ipairs({
    'echo foo',
    'echo  foo',
    'echo foo bar',
    'echo foo  bar',
    'echo "foo bar"',
    "echo 'foo|bar'",
    'echo "foo|bar"',
    'echo foo|echo bar',
    'echo foo | echo bar',
    'echo foo \\| echo bar',
    'echo foo " a comment',
    'echo foo "not a comment',
    'echo foo\\ bar',
    'echo foo\\\\bar',
    'echo foo\tbar',
    'echo ',
    'echo',
    'echo\t',
    -- EX_NOTRLCOM: the bar and the quote are literal argument bytes.
    'map a b',
    'map a b  c',
    'map a b|c',
    'map a b" c',
    'map <F2> :echo "hi"<CR>',
    'map',
    'map a',
    'nnoremap <buffer> x y',
    'abbreviate foo bar baz',
    'normal! ddp',
    'normal ix<Esc>|echo 1',
    -- EX_NOSPC: one argument, whitespace and all.
    'edit foo bar',
    'edit foo\\ bar',
    'file a b',
    'lcd some dir',
    -- EX_TRLBAR off: everything to the end of line.
    'echo 1 " two',
    'let x = 1 | let y = 2',
    'let x = "a|b"',
    'execute "ls"|edit foo',
    'autocmd BufRead * echo 1 | echo 2',
    'command! Foo echo 1 | echo 2',
    'function! Foo() abort',
    'syntax match Foo /x/ | echo 1',
    'highlight Foo guifg=red | echo 1',
    -- A trailing comment on a command that accepts one.
    'ls " listing',
    'ls! " listing',
    'set nu " a comment',
    'set nu| set nonu',
    -- Bars inside a pattern.
    's/a\\|b/c/',
    'g/a\\|b/d',
    -- Leading colons and whitespace.
    ':echo 1',
    ':::echo 1',
    '  echo 1',
    '\techo 1',
    ' : echo 1',
    -- Backslash-escaped bar in the :normal argument.
    'normal! A\\|<Esc>',
  }) do
    parse('c06/' .. tag(line), line)
  end
end)

-- ---------------------------------------------------------------------
-- s07 -- file arguments and the expandables.
-- ---------------------------------------------------------------------

section('s07-expand', function()
  for _, line in ipairs({
    'edit %',
    'edit #',
    'edit ##',
    'edit %:p',
    'edit %:p:h',
    'edit %:t:r',
    'edit <cword>',
    'edit <cWORD>',
    'edit <cfile>',
    'edit <afile>',
    'edit <abuf>',
    'edit <sfile>',
    'edit <slnum>',
    'edit \\%',
    'edit a%b',
    'edit *.txt',
    'edit ~/x',
    'edit $HOME/x',
    'write %',
    'read %',
    'source %',
    'argadd %',
    'echo %',
    'echo expand("%")',
    'grep <cword> %',
    'saveas %',
    'sfind x',
    'find x',
  }) do
    parse('c07/' .. tag(line), line)
  end
  -- The same lines executed with magic.file forced on and off, against
  -- a real (sandboxed) buffer name, so that the *expansion* is the
  -- answer rather than the parse.  `:file` renames the scratch buffer
  -- without touching the disk.
  for _, spec in ipairs({
    { 'edit', '%' },
    { 'edit', '%:t' },
    { 'edit', '<cword>' },
    { 'argadd', '%' },
    { 'badd', '%' },
  }) do
    for _, magic in ipairs({ true, false }) do
      local lbl = string.format('c07m/%s/%s/%s', spec[1], tag(spec[2]), tostring(magic))
      exec(lbl, nil, {
        pre = 'silent file ' .. work .. '/files/named.txt',
        dict = {
          cmd = spec[1],
          args = { spec[2] },
          magic = { file = magic, bar = true },
        },
        after = 'expand("%:t")',
      })
    end
  end
end)

-- ---------------------------------------------------------------------
-- s08 -- +cmd and ++opt.
-- ---------------------------------------------------------------------

section('s08-argopt', function()
  for _, line in ipairs({
    'edit +5 file.txt',
    'edit +1 file.txt',
    'edit + file.txt',
    'edit +$ file.txt',
    'edit +/pat file.txt',
    'edit +/Line\\ 2 file.txt',
    'edit +set\\ nomodifiable file.txt',
    'edit ++ff=mac file.txt',
    'edit ++fileformat=unix file.txt',
    'edit ++ff=dos ++enc=latin1 file.txt',
    'edit ++enc=utf-8 file.txt',
    'edit ++encoding=cp1252 file.txt',
    'edit ++bin file.txt',
    'edit ++nobin file.txt',
    'edit ++binary file.txt',
    'edit ++bad=keep file.txt',
    'edit ++bad=drop file.txt',
    'edit ++bad=? file.txt',
    'edit ++edit file.txt',
    'edit ++p file.txt',
    'edit ++ff=bogus file.txt',
    'edit ++bogus file.txt',
    'edit ++ file.txt',
    'edit ++ff= file.txt',
    'read ++ff=dos file.txt',
    'write ++ff=dos file.txt',
    'split +5 file.txt',
    'botright split ++ff=mac +3 file.txt',
    'argadd ++ff=mac file.txt',
    'source ++clear file.txt',
    'edit +5',
    'edit ++ff=mac',
  }) do
    parse('c08/' .. tag(line), line)
  end
  -- Parsing is not enough, and this cost two NOT-CAUGHT mutations:
  -- `nvim_parse_cmd` does NOT run `getargopt`.  It leaves `++bin` and
  -- `++bad=drop` sitting in `args` verbatim, so the parse side of this
  -- section gates the argument *splitter* and nothing in argopt.rs.
  -- These cases execute, against a fixture with CRLF line endings and an
  -- invalid byte, and read back what the options became.
  local AOPRE = table.concat({
    'call writefile(["alpha\\r", "beta\\r", "\\xffgamma\\r"], "files/ao.txt", "b")',
    'silent! call delete("files/aodir", "rf")',
  }, ' | ')
  local AOAFTER = 'string([&fileformat, &binary, &fileencoding, &modifiable, '
    .. 'line("."), getline(1), line("$")])'
  for _, line in ipairs({
    'edit ++ff=unix files/ao.txt',
    'edit ++ff=dos files/ao.txt',
    'edit ++ff=mac files/ao.txt',
    'edit ++fileformat=dos files/ao.txt',
    'edit ++bin files/ao.txt',
    'edit ++nobin files/ao.txt',
    'edit ++binary files/ao.txt',
    'edit ++nobinary files/ao.txt',
    'edit ++bad=keep files/ao.txt',
    'edit ++bad=drop files/ao.txt',
    'edit ++bad=? files/ao.txt',
    'edit ++bad=X files/ao.txt',
    'edit ++bad=zz files/ao.txt',
    'edit ++enc=latin1 files/ao.txt',
    'edit ++encoding=utf-8 files/ao.txt',
    'edit ++edit files/ao.txt',
    'edit ++editx files/ao.txt',
    'edit ++p files/aodir/sub.txt',
    'edit ++px files/ao.txt',
    'edit ++bogus files/ao.txt',
    'edit ++ff=bogus files/ao.txt',
    'edit ++ files/ao.txt',
    'edit ++ff= files/ao.txt',
    'edit ++ff=dos ++bin files/ao.txt',
    'edit +2 files/ao.txt',
    'edit +$ files/ao.txt',
    'edit +/beta files/ao.txt',
    'edit +set\\ nomodifiable files/ao.txt',
    'read ++ff=dos files/ao.txt',
    'read ++bin files/ao.txt',
    'read ++bad=drop files/ao.txt',
  }) do
    exec('c08x/' .. tag(line), line, { pre = AOPRE, after = AOAFTER })
  end
end)

-- ---------------------------------------------------------------------
-- s09 -- user commands.
-- ---------------------------------------------------------------------

local UC_ATTRS = {
  '',
  '-nargs=0',
  '-nargs=1',
  '-nargs=*',
  '-nargs=?',
  '-nargs=+',
  '-bang',
  '-bar',
  '-register',
  '-buffer',
  '-keepscript',
  '-range',
  '-range=%',
  '-range=5',
  '-count',
  '-count=3',
  '-addr=lines',
  '-addr=arguments',
  '-addr=buffers',
  '-addr=loaded_buffers',
  '-addr=windows',
  '-addr=tabs',
  '-addr=quickfix',
  '-addr=other',
  -- Strict *prefixes* of the table's names.  Every full spelling above
  -- resolves the same whether the lookup compares lengths or only
  -- prefixes, so without these the `-addr=` matcher is untestable: the
  -- baseline rejects all four with E180 and a prefix-matching mutant
  -- accepts them.
  '-addr=line',
  '-addr=arg',
  '-addr=buf',
  '-addr=win',
  '-complete=file',
  '-complete=buffer',
  '-complete=command',
  '-complete=custom,CsCustom',
  '-complete=customlist,CsCustomList',
  '-complete=dir -nargs=1',
  '-nargs=* -range -bang -register -bar',
  '-nargs=? -count=2 -addr=lines',
  '-range -addr=windows -nargs=*',
}

section('s09-usercmd', function()
  quiet([[
    function! CsCustom(A, L, P) abort
      return "aa\nbb\ncc"
    endfunction
    function! CsCustomList(A, L, P) abort
      return ['xx', 'yy']
    endfunction
    let g:cs_log = []
    function! CsSink(...) abort
      call add(g:cs_log, a:000)
    endfunction
  ]])
  for i, attr in ipairs(UC_ATTRS) do
    local name = string.format('Cs%02d', i)
    local def = string.format(
      'command! %s %s call CsSink(<q-args>)',
      attr,
      name
    )
    local err = quiet(def)
    emit('c09def/' .. tag(attr), err and ('E ' .. esc(err)) or 'ok')
    for _, invocation in ipairs({
      name,
      name .. '!',
      name .. ' one',
      name .. ' one two',
      '1,3' .. name,
      '%' .. name,
      '5' .. name,
      name .. ' a b c',
    }) do
      parse('c09p/' .. tag(attr) .. '/' .. tag(invocation), invocation)
    end
  end
  -- The replacement-string substitutions, each asked on its own so a
  -- broken one names itself.
  local SUBS = {
    '<args>',
    '<q-args>',
    '<f-args>',
    '<lt>args>',
    '<line1>',
    '<line2>',
    '<range>',
    '<count>',
    '<bang>',
    '<q-bang>',
    '<reg>',
    '<register>',
    '<mods>',
    '<q-mods>',
    '<amatch>',
    '<sfile>',
    '<slnum>',
    '<unknown>',
  }
  for i, sub in ipairs(SUBS) do
    local name = string.format('Csr%02d', i)
    quiet(
      string.format(
        'command! -nargs=* -range -count=7 -bang -register %s let g:cs_sub = "%s"',
        name,
        sub:gsub('"', '\\"')
      )
    )
    exec('c09s/' .. tag(sub), '1,3' .. name .. '! r alpha beta', {
      after = 'get(g:, "cs_sub", "<unset>")',
      noparse = false,
    })
  end
  -- Listing, redefining, deleting.
  exec('c09l/list', 'command', { after = '1' })
  exec('c09l/listone', 'command Cs01', { after = '1' })
  exec('c09l/redef', 'command! -nargs=1 Cs01 echo 1', { after = '1' })
  exec('c09l/redef-noforce', 'command -nargs=1 Cs01 echo 1', { after = '1' })
  exec('c09l/del', 'delcommand Cs01', { after = '1' })
  exec('c09l/del-missing', 'delcommand NoSuchCs', { after = '1' })
  exec('c09l/bad-attr', 'command! -bogus CsBad echo 1', { after = '1' })
  exec('c09l/bad-nargs', 'command! -nargs=9 CsBad echo 1', { after = '1' })
  exec('c09l/bad-name', 'command! lower echo 1', { after = '1' })
  exec('c09l/bad-addr', 'command! -addr=bogus CsBad echo 1', { after = '1' })
  exec('c09l/bad-complete', 'command! -complete=bogus CsBad echo 1', { after = '1' })
  exec('c09l/complete-nargs0', 'command! -complete=file CsBad echo 1', { after = '1' })
  -- Invoking, for real, against the argument splitter.
  for _, inv in ipairs({
    'Csr01 one two',
    'Csr01 one\\ two',
    'Csr01 "one two"',
    'Csr01 a\\|b',
    'Csr03 one two three',
    'Csr03 one\\ two three',
  }) do
    exec('c09i/' .. tag(inv), inv, { after = 'get(g:, "cs_sub", "<unset>")' })
  end
  -- Buffer-local commands, and the buffer/global collision.
  exec('c09b/local', 'command! -buffer CsLocal echo 1', { after = '1' })
  quiet('silent! comclear')
end)

-- ---------------------------------------------------------------------
-- s10 -- :substitute.
-- ---------------------------------------------------------------------

section('s10-subst', function()
  local SUBS = {
    '%s/alpha/ALPHA/',
    '%s/alpha/ALPHA/g',
    '%s/alpha/ALPHA/gn',
    '%s/alpha/ALPHA/n',
    '%s/alpha/ALPHA/e',
    '%s/nomatch/X/e',
    '%s/nomatch/X/',
    '%s/alpha/ALPHA/i',
    '%s/ALPHA/x/I',
    '%s/alpha/ALPHA/&',
    '%s/alpha/ALPHA/p',
    '%s/alpha/ALPHA/#',
    '%s/alpha/ALPHA/l',
    '%s/alpha/ALPHA/r',
    '%s#alpha#ALPHA#',
    '%s,alpha,ALPHA,',
    '%s+alpha+ALPHA+',
    '%s|alpha|ALPHA|',
    '%s@alpha@ALPHA@',
    '%s/alpha/ALPHA',
    '%s/alpha/',
    '%s/alpha',
    '%s//X/',
    '%s/L\\(%d\\)/[&]/',
    '%s/L0\\(\\d\\)/<\\1>/',
    '%s/beta/\\=toupper(submatch(0))/',
    '%s/beta/\\="x" . 1/',
    '%s/beta/\\r/',
    '%s/beta/a\\nb/',
    '%s/beta/~/',
    '%s/^/> /',
    '%s/$/ </',
    '%s/\\s\\+$//',
    '1,5s/alpha/A/',
    '5,1s/alpha/A/',
    '.s/alpha/A/',
    '$s/alpha/A/',
    "'a,'bs/alpha/A/",
    '%smagic/a.pha/X/',
    '%snomagic/a.pha/X/',
    '%s/a.pha/X/',
    '%sm/alpha/X/',
    '%sno/alpha/X/',
    '%substitute/alpha/X/',
    '%s/alpha/X/ 3',
    '%s/alpha/X/g 3',
    'substitute',
    's',
    '&',
    '&&',
    '~',
    'g&',
  }
  for _, line in ipairs(SUBS) do
    parse('c10p/' .. tag(line), line)
    exec('c10x/' .. tag(line), line, { after = 'getreg("/")' })
  end
  -- The repeat forms need a previous :s, which is what `pre` seeds.
  for _, line in ipairs({ 's', '&', '&&', '~', 'g&', '%s//Y/', '%s/beta//' }) do
    exec('c10r/' .. tag(line), line, {
      pre = 'silent 1,3s/alpha/Q/g',
      after = 'getreg("/")',
    })
  end

  -- B17-5.  `do_sub` is 1,220 lines and the fifty-odd cases above are the
  -- whole of its differential coverage; these five blocks are the arms they
  -- never reached.

  -- c10c -- the `c` (confirm) flag.  The prompt reads a character, and in
  -- `-l` script mode a character fed with `feedkeys(..., 'n')` is what it
  -- gets: the answers below drive every arm of the dialogue.  The PROMPT
  -- ITSELF goes to stderr, so this block is also the only view of
  -- do_sub's confirm message.  Feed EXACTLY the keys a case consumes:
  -- typeahead survives the command, and a leftover key would answer the
  -- next case's first prompt.  `no-keys` is deliberate -- stdin is an
  -- empty file, so the prompt reads EOF, which is its own arm.
  local CFIX = { 'alpha one alpha', 'beta two', 'alpha three', 'alpha four alpha' }
  for _, c in ipairs({
    { 'all-y', 'yyy', '%s/alpha/X/c' },
    { 'all-n', 'nnn', '%s/alpha/X/c' },
    { 'y-n-y', 'yny', '%s/alpha/X/c' },
    { 'quit-first', 'q', '%s/alpha/X/c' },
    { 'quit-second', 'yq', '%s/alpha/X/c' },
    { 'last', 'l', '%s/alpha/X/c' },
    { 'y-then-last', 'yl', '%s/alpha/X/c' },
    { 'all-after-one', 'ya', '%s/alpha/X/c' },
    { 'g-all-y', 'yyyyy', '%s/alpha/X/gc' },
    { 'g-mixed', 'ynyny', '%s/alpha/X/gc' },
    { 'g-all-after-two', 'yna', '%s/alpha/X/gc' },
    { 'g-quit', 'ynq', '%s/alpha/X/gc' },
    { 'esc-then-quit', '\27q', '%s/alpha/X/c' },
    { 'ctrl-e-then-yq', '\5yq', '%s/alpha/X/c' },
    { 'count-c', 'yy', '1,3s/alpha/X/c' },
    { 'ce-nomatch', '', '%s/nomatch/X/ce' },
  }) do
    exec('c10c/' .. c[1], c[3], {
      lines = CFIX,
      pre = c[2] ~= '' and ('call feedkeys(' .. vim.fn.string(c[2]) .. ', "n")') or nil,
      after = 'string([getreg("/"), line("$"), getline(1)])',
    })
    -- EVERY case above must feed exactly as many answers as its prompt asks
    -- for, and a case that runs OUT is not a failed case: the prompt reads
    -- the real input stream, which is an empty file, and nvim `exit(0)`s
    -- mid-run -- the whole sweep stops there and the report simply ends,
    -- with status 0.  (Same family as `:debug` under `-l`; P0.4b.)  Neither
    -- `<Esc>` nor `<C-E>` answers the prompt, they re-ask it, which is why
    -- both carry a terminating key.  This drains anything left over without
    -- EXECUTING it -- an unconsumed `q` would otherwise start a recording
    -- in the next case's `reset` -- and reports the count, so a leak is a
    -- diff rather than a mystery.
    quiet(
      'let g:csdrain = 0 | while g:csdrain < 200 | let g:c = getchar(0) '
        .. '| if type(g:c) == v:t_number && g:c == 0 | break | endif '
        .. '| let g:csdrain += 1 | endwhile | unlet! g:c'
    )
    local dok, drained = pcall(vim.fn.eval, 'g:csdrain')
    if not dok or drained ~= 0 then
      emit('c10c/' .. c[1], 'A!', 'typeahead-left=' .. esc(tostring(drained)))
    end
  end

  -- c10f -- the repeat forms, as a MATRIX rather than a list.  `:&` reuses
  -- the pattern and drops the flags, `:&&` keeps them, `:~` reuses the last
  -- SEARCH pattern instead of the last substitute pattern, and `g&` is
  -- `:%s//~/&` -- four different answers to "what does 'again' mean", and
  -- the previous command is what separates them.
  for _, prev in ipairs({
    { 'plain', 'silent 1,4s/alpha/Q/' },
    { 'gflag', 'silent 1,4s/alpha/Q/g' },
    { 'iflag', 'silent 1,4s/ALPHA/Q/gi' },
    { 'search', 'silent 1,4s/alpha/Q/ | let @/ = "beta"' },
    { 'nflag', 'silent 1,4s/alpha/Q/gn' },
  }) do
    for _, line in ipairs({
      '&', '&&', '~', 's', 'sg', 's g', 'g&', '2&', '2&&', '%&&', '%~',
      '5,8&&', 's//NEW/', 's/beta//', '&&e', '&e', '2,3~',
    }) do
      exec('c10f/' .. prev[1] .. '/' .. tag(line), line, {
        pre = prev[2],
        after = 'string([getreg("/"), getline(1), getline(2)])',
      })
    end
  end

  -- c10e -- `\=`, which re-enters the evaluator from inside the per-line
  -- match loop.  The interesting arms are the ones that come back with
  -- something other than a plain string: a List (one line per element), a
  -- newline inside a String, an error, and a call that tries to change the
  -- buffer that do_sub is halfway through rewriting.
  for _, line in ipairs({
    '%s/alpha/\\=submatch(0) . "!"/',
    '%s/alpha/\\=toupper(submatch(0))/g',
    '%s/L\\(\\d\\d\\)/\\=str2nr(submatch(1)) * 2/',
    '%s/\\(a\\)\\(l\\)/\\=submatch(2) . submatch(1)/g',
    '%s/alpha/\\=line(".")/',
    '%s/alpha/\\=printf("%s-%d", submatch(0), 7)/',
    '%s/alpha/\\=[1,2]/',
    '%s/alpha/\\=["x"]/',
    '%s/alpha/\\=[]/',
    '%s/alpha/\\="a\\nb"/',
    '%s/alpha/\\="a\\rb"/',
    '%s/alpha/\\=v:null/',
    '%s/alpha/\\=7/',
    '%s/alpha/\\=1.5/',
    '%s/alpha/\\={"k": 1}/',
    '%s/alpha/\\=no_such_function_here()/',
    '%s/alpha/\\=/',
    '%s/alpha/\\=setline(1, "hijacked")/',
    '%s/alpha/\\=execute("echo 1")/',
    '%s/alpha/\\=submatch(9)/',
    '%s/alpha/\\=string(submatch(0, 1))/',
    '1,3s/alpha/\\=submatch(0)/gn',
    '%s/alpha/\\=g:undefined_var/',
  }) do
    exec('c10e/' .. tag(line), line, { after = 'string([getreg("/"), line("$")])' })
  end

  -- c10m -- patterns and replacements that cross a line boundary.  `\n` in
  -- the pattern makes one match span two lines, which is the arm of do_sub
  -- that deletes lines rather than replacing text; `\r` in the replacement
  -- splits one line into two; a literal NL in the replacement inserts a NUL.
  for _, line in ipairs({
    '%s/beta\\nL02/JOINED/',
    '1,3s/\\n//',
    '1,3s/\\n/+/',
    '%s/alpha beta\\nL02 alpha/X/',
    '%s/alpha/a\\rb/',
    '%s/alpha/a\\nb/',
    '1,2s/\\(L01\\)\\_.\\{-}\\(L02\\)/\\2-\\1/',
    '%s/\\%^L01/START/',
    '%s/L20 alpha beta\\%$/END/',
    '%s/beta\\n//',
    '%s/\\nL/ L/g',
    '1,4s/^/\\r/',
    '%s/e\\ns/E-S/',
    '%s/\\_s\\+/ /g',
  }) do
    exec('c10m/' .. tag(line), line, { after = 'string([line("$"), getreg("/")])' })
  end

  -- c10i -- 'inccommand'.  Through nvim_cmd there is no command line and so
  -- no preview, which is exactly the assertion: setting 'inccommand' must
  -- not change what a *completed* `:s` does.  ex_substitute_preview itself
  -- is driven from s18, which needs a command line.
  for _, icm in ipairs({ '', 'nosplit', 'split' }) do
    for _, line in ipairs({
      '%s/alpha/X/', '%s/alpha/X/g', '%s/alpha/X/gc', '%s/alpha/\\=1+1/',
      '%s/nomatch/X/', 'g/alpha/s//Y/',
    }) do
      exec('c10i/' .. (icm == '' and 'off' or icm) .. '/' .. tag(line), line, {
        opts = 'inccommand=' .. icm,
        pre = line:find('c$') and 'call feedkeys("a", "n")' or nil,
        after = 'string([&inccommand, getreg("/")])',
      })
    end
  end
  for _, line in ipairs({ 'set inccommand=bogus', 'set inccommand=', 'set icm?' }) do
    exec('c10i/opt/' .. tag(line), line, { after = '&inccommand' })
  end
end)

-- ---------------------------------------------------------------------
-- s11 -- :global.
-- ---------------------------------------------------------------------

section('s11-global', function()
  local GLOBALS = {
    'g/alpha/d',
    'g!/alpha/d',
    'v/alpha/d',
    'g/MATCH/p',
    'g/MATCH/normal! A!',
    'g/L0/s/alpha/A/',
    'g/L1/s//X/',
    'g#alpha#d',
    'g,alpha,d',
    '1,5g/alpha/d',
    'g/^$/d',
    'g/alpha/',
    'g//d',
    'g/nomatch/d',
    'global/alpha/d',
    'vglobal/alpha/d',
    'g/alpha/g/beta/d',
    'g/L0/normal! dd',
    'g/L1/t$',
    'g/L1/m0',
    'g/alpha/echo 1',
    'g/alpha/d | echo 2',
  }
  for _, line in ipairs(GLOBALS) do
    parse('c11p/' .. tag(line), line)
    exec('c11x/' .. tag(line), line)
  end

  -- B17-5.  `ex_global` re-enters `do_cmdline` once per marked line, so the
  -- risk it carries is entirely re-entrancy: what the body does to the lines
  -- that are still marked, and what a second `:g` inside the first does to
  -- the mark set.  None of that is reachable from the flat list above.
  for _, line in ipairs({
    -- nested, two and three deep
    'g/L0/g/alpha/s//X/',
    'g/alpha/g/beta/g/L/d',
    'g/L1/v/alpha/d',
    'v/MATCH/g/beta/s/beta/B/',
    'g/alpha/g!/beta/d',
    -- a body that moves the lines the outer :g still has marked
    'g/alpha/m$',
    'g/alpha/m0',
    'g/alpha/t0',
    'g/alpha/t$',
    'g/L0/normal! ddp',
    'g/alpha/normal! J',
    'g/alpha/-1d',
    'g/alpha/+1d',
    'g/alpha/.,+1d',
    -- a body that is itself a range-carrying :s
    'g/alpha/.,+1s/alpha/Y/',
    'g/alpha/s//&&/',
    'g/alpha/s/alpha/\\=line(".")/',
    'g/L0/1,$s/beta/B/',
    -- control flow and re-entrant execution in the body
    'g/alpha/if line(".") % 2 | s/alpha/Z/ | endif',
    'g/alpha/exe "s/alpha/E/"',
    'g/alpha/normal! @q',
    'g/alpha/call setline(".", "replaced")',
    'g/alpha/undo',
    'g/alpha/g/alpha/undo',
    -- the pattern arms
    'v//d',
    'g/\\%^/d',
    'g/\\%$/d',
    'g/alpha\\nL02/d',
    'g/\\(/d',
    'g/alpha/nosuchcommand',
    'g/alpha/s/\\(/X/',
    'global!/alpha/s/beta/B/',
    -- ranges and counts
    '2,4g/alpha/d',
    '.,$g/alpha/s//X/',
    "'a,'bg/alpha/d",
    '2,4v/MATCH/s/alpha/V/',
  }) do
    parse('c11np/' .. tag(line), line)
    exec('c11nx/' .. tag(line), line, { after = 'string([line("$"), getreg("/"), getline(1)])' })
  end
end)

-- ---------------------------------------------------------------------
-- s12 -- control flow, parse side.
-- ---------------------------------------------------------------------

section('s12-flow', function()
  for _, line in ipairs({
    'if 1',
    'if v:true',
    'elseif 0',
    'else',
    'endif',
    'en',
    'while 1',
    'endwhile',
    'endw',
    'for i in [1,2]',
    'endfor',
    'endfo',
    'continue',
    'break',
    'try',
    'catch',
    'catch /E\\d\\+/',
    'catch /^Vim\\%((\\a\\+)\\)\\=:E/',
    'finally',
    'endtry',
    'endt',
    'throw "x"',
    'throw',
    'function Foo()',
    'function! Foo() abort',
    'function',
    'function /Foo',
    'delfunction Foo',
    'return',
    'return 1',
    'call Foo()',
    'execute "echo 1"',
    'eval 1',
    'silent! throw "x"',
    'if 1 | echo 2 | endif',
    'try | throw "x" | catch | endtry',
  }) do
    parse('c12/' .. tag(line), line)
  end
  -- Executed bodies, handed over in one nvim_exec2 so that a definition
  -- never asks the real input stream (which in `-l` mode exits(0)).
  local BODIES = {
    ['if-true'] = 'if 1\nlet g:cs_f = "then"\nelse\nlet g:cs_f = "else"\nendif',
    ['if-false'] = 'if 0\nlet g:cs_f = "then"\nelse\nlet g:cs_f = "else"\nendif',
    ['elseif'] = 'if 0\nlet g:cs_f="a"\nelseif 1\nlet g:cs_f="b"\nelse\nlet g:cs_f="c"\nendif',
    ['while'] = 'let g:cs_f = 0\nwhile g:cs_f < 3\nlet g:cs_f += 1\nendwhile',
    ['for'] = 'let g:cs_f = []\nfor i in [1,2,3]\ncall add(g:cs_f, i)\nendfor',
    ['for-break'] = 'let g:cs_f = []\nfor i in [1,2,3]\nif i == 2 | break | endif\ncall add(g:cs_f, i)\nendfor',
    ['for-continue'] = 'let g:cs_f = []\nfor i in [1,2,3]\nif i == 2 | continue | endif\ncall add(g:cs_f, i)\nendfor',
    ['try-throw'] = 'let g:cs_f = ""\ntry\nthrow "boom"\ncatch /boom/\nlet g:cs_f = "caught:" . v:exception\nfinally\nlet g:cs_f .= "|fin"\nendtry',
    ['try-error'] = 'let g:cs_f = ""\ntry\nQwertyNoSuch\ncatch\nlet g:cs_f = v:exception\nendtry',
    ['try-nested'] = 'let g:cs_f = ""\ntry\ntry\nthrow "in"\nfinally\nlet g:cs_f .= "f1"\nendtry\ncatch\nlet g:cs_f .= "|c:" . v:exception\nendtry',
    ['try-rethrow'] = 'let g:cs_f = ""\ntry\ntry\nthrow "a"\ncatch\nthrow "b"\nendtry\ncatch\nlet g:cs_f = v:exception\nendtry',
    ['try-finally-return'] = 'function! CsTf() abort\ntry\nreturn 1\nfinally\nlet g:cs_f = "fin"\nendtry\nendfunction\nlet g:cs_r = CsTf()',
    ['uncaught'] = 'let g:cs_f = "before"\nthrow "loose"',
    ['catch-all'] = 'let g:cs_f = ""\ntry\ncall NoSuchFn()\ncatch /E117/\nlet g:cs_f = "E117"\nendtry',
    ['throw-in-catch'] = 'let g:cs_f = ""\ntry\nthrow "x"\ncatch\nlet g:cs_f = v:throwpoint != "" ? "haspoint" : "nopoint"\nendtry',
  }
  local names = {}
  for k in pairs(BODIES) do
    names[#names + 1] = k
  end
  table.sort(names)
  for _, name in ipairs(names) do
    reset()
    quiet('unlet! g:cs_f g:cs_r')
    local err = quiet(BODIES[name])
    emit('c12x/' .. name, 'R', err and ('E ' .. esc(err)) or 'ok')
    local ok, val = pcall(vim.fn.eval, 'string([get(g:,"cs_f","<unset>"), get(g:,"cs_r","<unset>")])')
    emit('c12x/' .. name, 'A', ok and esc(scrub(val)) or esc(errtext(val)))
    struct('c12x/' .. name, { err = err, a = ok and scrub(val) or nil })
  end
  quiet('silent! delfunction CsTf')
end)

-- ---------------------------------------------------------------------
-- s13 -- the ex_cmds.rs shell, executed.
-- ---------------------------------------------------------------------

section('s13-excmds', function()
  local CASES = {
    '1,3delete',
    '1,3delete a',
    '1,3delete a 5',
    '$delete',
    '0,0delete',
    '1delete 3',
    '1,3yank',
    '1,3yank b',
    '3put',
    '3put!',
    '0put',
    '$put',
    'put =[1,2,3]',
    'put ="x"',
    '1,3move 10',
    '1,3move 0',
    '1,3move $',
    '1,3move 2',
    '1,3copy 10',
    '1,3copy 0',
    '1,3t$',
    '1,3t.',
    '1,3join',
    '1,3join!',
    'join',
    '5,8join',
    '1,3>',
    '1,3<',
    '1,3>>',
    '1,3<<',
    '1,3> 2',
    '1,3normal! A!',
    '1,3normal Ix',
    '1,20sort',
    '1,20sort!',
    '1,20sort u',
    '1,20sort n',
    '1,20sort /L\\d\\d/',
    '1,20sort i',
    '1,3print',
    '1,3number',
    '1,3list',
    '1,3#',
    '1,3p',
    '5z',
    'z 3',
    'z=3',
    '1,3d|1,3p',
    'undo',
    '1,3left',
    '1,3left 4',
    '1,3right 20',
    '1,3center 20',
    '1,3retab',
    '1,3retab!',
    '12retab 4',
    '1,3s/a/b/',
    'file newname.txt',
    'file',
    '0file',
    'keepalt file kept.txt',
    '1,3d _',
    'silent 1,3d',
    'silent! 1,3d',
    'lockmarks 1,3d',
    'keepjumps 1,3d',
    'noautocmd 1,3d',
    'sandbox echo 1',
    'sandbox 1,3d',
    'browse echo 1',
    'confirm echo 1',
  }
  for _, line in ipairs(CASES) do
    exec('c13/' .. tag(line), line, { pos = { 4, 0 }, after = 'string([line("."), line("$")])' })
    exec('c13e/' .. tag(line), line, {
      pos = { 4, 0 },
      via = 'exec2',
      noparse = true,
      after = 'string([line("."), line("$")])',
    })
  end
end)

-- ---------------------------------------------------------------------
-- s14 -- nvim_cmd over hand-built Dicts.
-- ---------------------------------------------------------------------

section('s14-dict', function()
  local CASES = {
    { 'empty', {} },
    { 'cmd-only', { cmd = 'echo' } },
    { 'cmd-empty', { cmd = '' } },
    { 'cmd-empty-range', { cmd = '', range = { 3 } } },
    { 'cmd-empty-mods', { cmd = '', mods = { silent = true } } },
    { 'cmd-empty-count', { cmd = '', count = 3 } },
    { 'cmd-unknown', { cmd = 'Qwerty' } },
    { 'cmd-notimpl', { cmd = 'gui' } },
    { 'cmd-abbrev', { cmd = 'ec', args = { '1' } } },
    { 'cmd-number', { cmd = 42 } },
    { 'cmd-list', { cmd = { 'echo' } } },
    { 'args-string', { cmd = 'echo', args = 'foo' } },
    { 'args-number', { cmd = 'echo', args = { 1, 2 } } },
    { 'args-nested', { cmd = 'echo', args = { { 'x' } } } },
    { 'args-toomany', { cmd = 'edit', args = { 'a', 'b' } } },
    { 'args-toofew', { cmd = 'edit', args = {} } },
    { 'args-none-given', { cmd = 'ls', args = { 'x' } } },
    { 'bang-ok', { cmd = 'write', args = { 'files/w1.txt' }, bang = true } },
    { 'bang-refused', { cmd = 'echo', args = { '1' }, bang = true } },
    { 'bang-number', { cmd = 'echo', args = { '1' }, bang = 1 } },
    { 'range-empty', { cmd = 'print', range = {} } },
    { 'range-one', { cmd = 'print', range = { 3 } } },
    { 'range-two', { cmd = 'print', range = { 2, 4 } } },
    { 'range-three', { cmd = 'print', range = { 1, 2, 3 } } },
    { 'range-neg', { cmd = 'print', range = { -1 } } },
    { 'range-huge', { cmd = 'print', range = { 1, 999999 } } },
    { 'range-float', { cmd = 'print', range = { 1.5 } } },
    { 'range-string', { cmd = 'print', range = { '1' } } },
    { 'range-refused', { cmd = 'echo', args = { '1' }, range = { 1, 2 } } },
    { 'count-ok', { cmd = 'buffer', count = 1 } },
    { 'count-refused', { cmd = 'echo', args = { '1' }, count = 3 } },
    { 'count-neg', { cmd = 'buffer', count = -1 } },
    { 'count-string', { cmd = 'buffer', count = 'x' } },
    { 'reg-ok', { cmd = 'delete', range = { 1, 2 }, reg = 'a' } },
    { 'reg-refused', { cmd = 'echo', args = { '1' }, reg = 'a' } },
    { 'reg-long', { cmd = 'delete', range = { 1 }, reg = 'ab' } },
    { 'reg-empty', { cmd = 'delete', range = { 1 }, reg = '' } },
    { 'reg-blackhole', { cmd = 'delete', range = { 1 }, reg = '_' } },
    { 'reg-number', { cmd = 'delete', range = { 1 }, reg = 1 } },
    { 'nargs-given', { cmd = 'echo', args = { '1' }, nargs = '*' } },
    { 'addr-given', { cmd = 'print', range = { 1 }, addr = 'line' } },
    { 'nextcmd-given', { cmd = 'echo', args = { '1' }, nextcmd = 'echo 2' } },
    { 'magic-file-off', { cmd = 'echo', args = { '%' }, magic = { file = false } } },
    { 'magic-bar-off', { cmd = 'echo', args = { '1' }, magic = { bar = false } } },
    { 'magic-string', { cmd = 'echo', args = { '1' }, magic = 'x' } },
    { 'magic-unknown', { cmd = 'echo', args = { '1' }, magic = { bogus = true } } },
    { 'mods-silent', { cmd = 'echo', args = { '"m"' }, mods = { silent = true } } },
    { 'mods-emsg-silent', { cmd = 'Qwerty', mods = { emsg_silent = true } } },
    { 'mods-unknown', { cmd = 'echo', args = { '1' }, mods = { bogus = true } } },
    { 'mods-string', { cmd = 'echo', args = { '1' }, mods = 'silent' } },
    { 'mods-tab', { cmd = 'split', mods = { tab = 1 } } },
    { 'mods-tab-neg', { cmd = 'split', mods = { tab = -2 } } },
    { 'mods-tab-huge', { cmd = 'split', mods = { tab = 99 } } },
    { 'mods-verbose', { cmd = 'echo', args = { '1' }, mods = { verbose = 3 } } },
    { 'mods-verbose-neg', { cmd = 'echo', args = { '1' }, mods = { verbose = -2 } } },
    { 'mods-split', { cmd = 'split', mods = { split = 'botright' } } },
    { 'mods-split-bogus', { cmd = 'split', mods = { split = 'sideways' } } },
    { 'mods-vertical', { cmd = 'split', mods = { vertical = true } } },
    { 'mods-horizontal', { cmd = 'split', mods = { horizontal = true } } },
    { 'mods-filter', { cmd = 'ls', mods = { filter = { pattern = 'x', force = false } } } },
    { 'mods-filter-force', { cmd = 'ls', mods = { filter = { pattern = 'x', force = true } } } },
    { 'mods-filter-string', { cmd = 'ls', mods = { filter = 'x' } } },
    { 'mods-filter-badpat', { cmd = 'ls', mods = { filter = { pattern = '\\(' } } } },
    { 'mods-noautocmd', { cmd = 'echo', args = { '1' }, mods = { noautocmd = true } } },
    { 'mods-keepalt', { cmd = 'echo', args = { '1' }, mods = { keepalt = true } } },
    { 'mods-sandbox', { cmd = 'call', args = { 'setline(1,"x")' }, mods = { sandbox = true } } },
    { 'mods-lockmarks', { cmd = 'delete', range = { 1, 2 }, mods = { lockmarks = true } } },
    { 'mods-browse', { cmd = 'echo', args = { '1' }, mods = { browse = true } } },
    { 'mods-confirm', { cmd = 'echo', args = { '1' }, mods = { confirm = true } } },
    { 'mods-hide', { cmd = 'echo', args = { '1' }, mods = { hide = true } } },
    { 'mods-noswapfile', { cmd = 'echo', args = { '1' }, mods = { noswapfile = true } } },
    { 'mods-unsilent', { cmd = 'echo', args = { '"u"' }, mods = { unsilent = true } } },
    { 'mods-keepjumps', { cmd = 'normal', args = { 'G' }, mods = { keepjumps = true } } },
    { 'mods-keepmarks', { cmd = 'delete', range = { 1, 2 }, mods = { keepmarks = true } } },
    { 'mods-keeppatterns', { cmd = 'substitute', args = { '/alpha/X/' }, range = { 1, 3 }, mods = { keeppatterns = true } } },
  }
  for _, case in ipairs(CASES) do
    exec('c14/' .. case[1], nil, { dict = case[2], after = 'getreg("/")' })
  end
  -- The opts Dict itself.
  local OPTS = {
    { 'output-true', { output = true } },
    { 'output-false', { output = false } },
    { 'output-nil', {} },
    { 'output-number', { output = 1 } },
    { 'opts-unknown', { bogus = true } },
  }
  for _, o in ipairs(OPTS) do
    exec('c14o/' .. o[1], nil, { dict = { cmd = 'echo', args = { '"opt"' } }, copts = o[2] })
  end
end)

-- ---------------------------------------------------------------------
-- s15 -- parse then execute: the composition the API promises.
-- ---------------------------------------------------------------------

section('s15-roundtrip', function()
  local LINES = {
    'echo "rt"',
    'silent echo "rt"',
    '1,3delete',
    '1,3delete a',
    '3buffer',
    'set cursorline',
    'let g:cs_rt = 1',
    'normal! ddp',
    '%s/alpha/RT/g',
    'g/MATCH/d',
    'vertical botright split',
    '2tab split',
    'verbose 5 set nu',
    'filter /L0/ ls',
    'map a b',
    'edit ++ff=mac files/rt.txt',
    'edit +3 files/rt.txt',
    'lockmarks 1,2d',
    '1,3>',
    'argadd files/rt.txt',
  }
  for _, line in ipairs(LINES) do
    exec('c15/' .. tag(line), line, { after = 'string([line("."), line("$"), argc()])' })
  end
  -- The round trip with the Dict handed back EXACTLY as parsed.  Every
  -- EX_REGSTR command fails here on an empty `reg` -- the documented
  -- "modify the result of nvim_parse_cmd() then pass it to nvim_cmd()"
  -- does not hold for them.  These rows are the gate on that.
  for _, line in ipairs({
    '1,3delete',
    '1,3delete a',
    '1,3yank',
    'put',
    'put +',
    '3put!',
    'registers',
    'echo "rt"',
    '1,3print',
  }) do
    exec('c15raw/' .. tag(line), line, {
      rawreg = true,
      after = 'string([line("."), line("$")])',
    })
  end
end)

-- ---------------------------------------------------------------------
-- s16 -- deliberately malformed input.
-- ---------------------------------------------------------------------

section('s16-errors', function()
  local BAD = {
    '',
    ' ',
    '"',
    '" foo',
    '|',
    '||',
    ':',
    '::',
    'Fubar',
    'Fubar!',
    '4,6Fubar',
    'F',
    'fubar',
    'z!',
    'echo!',
    'ls!!',
    's/',
    's/a',
    's/\\(/x/',
    'g',
    'g/',
    'g/x',
    'v',
    'normal',
    'normal!',
    "'z,'yd",
    '/nomatchxyz/d',
    '?nomatchxyz?d',
    '1,2,3,4d',
    '.,,d',
    "'",
    "'@d",
    '$$$d',
    '+++++d',
    '9999999999999999999d',
    '-9999999999999999999d',
    '1,99999999999999999999d',
    'edit ++',
    'edit ++zzz',
    'edit +',
    'command',
    'command!',
    'delcommand',
    'delfunction',
    'unlet',
    'set nosuchoption',
    'setlocal nosuchoption',
    'let',
    'let =',
    'call',
    'call NoSuch(',
    'if',
    'elseif',
    'endif',
    'endwhile',
    'endfor',
    'endtry',
    'catch',
    'finally',
    'while',
    'for',
    'try!',
    'throw',
    'return',
    'break',
    'continue',
    'wincmd',
    'tabclose 99',
    'buffer 99999',
    'sleep -1',
    'k',
    'ka',
    'mark',
    'mark ab',
    "'a,'bmark",
    'sort /\\(/',
    'sort zz',
    'digraph',
    'digraph a',
    'digraph a: zz',
    'help',
    'lua',
    'lua =',
    'ruby 1',
    'perl 1',
    'python 1',
    'tcl 1',
    'mzscheme 1',
    'gui',
    'gvim',
    'shell',
    'stop',
    'browse!',
    'vertical!',
    'silent!!',
    'verbose!',
    'filter!',
    'filter! ',
    'tab!',
  }
  for _, line in ipairs(BAD) do
    parse('c16p/' .. tag(line), line)
  end
  -- The same corpus through nvim_cmd, where a parse failure is a
  -- *different* error from an execution failure.
  --
  -- `:let` with no argument is excluded: it lists *every* variable,
  -- including v:starttime (a nanosecond clock), v:servername (a fresh
  -- socket path), v:argv and b:changedtick -- four different kinds of
  -- irreproducible, and the variable layer is varsweep's question
  -- anyway.  The parse of it is still in c16p.
  local NOEXEC = { ['let'] = true }
  for _, line in ipairs(BAD) do
    if line ~= '' and not line:match('^%s*$') and not NOEXEC[line] then
      exec('c16x/' .. tag(line), line, { copts = { output = true } })
    end
  end
end)

-- ---------------------------------------------------------------------
-- s17 -- the rest of B17 through its own commands.
-- ---------------------------------------------------------------------

section('s17-b17', function()
  -- help.rs
  for _, line in ipairs({
    'help',
    'help :edit',
    'help E492',
    'help nosuchtopic123',
    'help!',
    'helpclose',
    'vertical help :edit',
    '3help :edit',
  }) do
    exec('c17h/' .. tag(line), line, { after = 'string([&buftype, bufname("%") =~ "help" ])' })
  end
  -- digraph.rs
  for _, line in ipairs({
    'digraphs',
    'digraph a: 228',
    'digraph a: 228 o: 246',
    'digraph!',
    'digraph zz 65',
    'digraph a',
    'digraph a: x',
  }) do
    exec('c17d/' .. tag(line), line, { after = 'digraph_get("a:")' })
  end
  -- cmdhist.rs
  exec('c17hi/history', 'history', {
    pre = 'call histadd(":", "echo one") | call histadd(":", "echo two") | call histadd("/", "pat")',
    after = 'string([histnr(":"), histget(":", -1), histget("/", -1)])',
  })
  for _, line in ipairs({
    'history :',
    'history /',
    'history all',
    'history search',
    'history cmd 1,2',
    'history =',
    'history @',
    'history >',
    'history bogus',
  }) do
    exec('c17hi/' .. tag(line), line, {
      pre = 'call histadd(":", "echo one") | call histadd(":", "echo two") | call histadd("/", "pat")',
      after = 'histnr(":")',
    })
  end
  -- arglist
  for _, line in ipairs({
    'args',
    'args files/a.txt files/b.txt',
    'argadd files/c.txt',
    'argadd',
    'argdelete',
    'argdelete files/a.txt',
    '%argdelete',
    'argument 1',
    'argument 99',
    'next',
    'previous',
    'first',
    'last',
    'rewind',
    'wnext',
    'argdo echo 1',
    'argglobal files/a.txt',
    'arglocal',
    'argedit files/d.txt',
  }) do
    exec('c17a/' .. tag(line), line, {
      pre = 'silent! args files/a.txt files/b.txt',
      after = 'string([argc(), argidx(), argv()])',
    })
  end
  -- ex_session.rs: the session file is a golden.
  for _, spec in ipairs({
    { 'plain', 'mksession! files/s1.vim', 'files/s1.vim' },
    { 'nosplit', 'mksession files/s2.vim', 'files/s2.vim' },
    { 'again', 'mksession files/s2.vim', 'files/s2.vim' },
    { 'view', 'mkview! files/v1.vim', 'files/v1.vim' },
    { 'exrc', 'mkexrc! files/e1.vim', 'files/e1.vim' },
    { 'vimrc', 'mkvimrc! files/r1.vim', 'files/r1.vim' },
  }) do
    exec('c17s/' .. spec[1], spec[2], {
      pre = 'silent! args files/a.txt files/b.txt | silent! only',
      after = string.format(
        'join(map(filereadable("%s") ? readfile("%s") : ["<none>"], '
          .. '{_, l -> substitute(l, "\\\\d\\\\+", "N", "g")}), "\\n")',
        spec[3],
        spec[3]
      ),
    })
  end
  -- debugger.rs
  for _, line in ipairs({
    'breakadd func 1 Foo',
    'breakadd file 3 files/a.txt',
    'breakadd here',
    'breakadd expr g:x',
    'breakadd',
    'breaklist',
    'breakdel 1',
    'breakdel *',
    'breakdel func Foo',
    'breakdel bogus',
  }) do
    -- `after` is the *listing*, not `1`.  A breakpoint is otherwise
    -- write-only from here: `:breaklist` was its own case, and every
    -- case's `pre` wiped the table first, so the listing was always
    -- empty and a mutation on which kind of breakpoint `:breakadd file`
    -- creates measured NOT CAUGHT.  The `pre` seeds two breakpoints so
    -- the delete cases have something to delete.
    exec('c17b/' .. tag(line), line, {
      pre = 'silent! breakdel * | silent! breakadd func 2 CsSeed '
        .. '| silent! breakadd file 4 files/b.txt',
      -- The breakpoint *number* is one counter for the whole process, so
      -- the raw listing would be a function of every case above this
      -- one.  The question is which breakpoints exist and of what kind.
      after = [[substitute(execute("breaklist"), '\v(^|\n)\s*\d+\s', '\1 N ', 'g')]],
    })
  end
  -- `:debug` is parse-only, and this is why: executing it enters the
  -- debugger, which asks the *real* input stream -- EOF in `-l` mode --
  -- and nvim exit(0)s mid-run, silently truncating the report.
  for _, line in ipairs({ 'debug echo 1', 'debug', 'debuggreedy', '0debuggreedy' }) do
    parse('c17b/' .. tag(line), line)
  end
  -- ex_cmds2.rs
  for _, line in ipairs({
    'scriptnames',
    'scriptencoding utf-8',
    'scriptencoding',
    'source files/src.vim',
    'source! files/src.vim',
    'source files/nosuch.vim',
    'runtime plugin/nosuch.vim',
    'runtime! plugin/nosuch.vim',
    'runtime START plugin/nosuch.vim',
    'finish',
    'language C',
    'language messages C',
    'language bogus',
  }) do
    exec('c17c/' .. tag(line), line, {
      pre = 'call writefile(["let g:cs_src = 1", "echo \\"sourced\\""], "files/src.vim")',
      after = 'get(g:, "cs_src", "<unset>")',
    })
  end

  -- B17-5.  `:helpgrep` and `:helptags` are the two halves of help.rs that
  -- none of its four gates reaches (`test_help` 16, `test_help_tagjump` 3,
  -- `help_spec` 318 and c17h above are all about `:help` itself).
  --
  -- Over a FIXTURE help tree, not `runtime/doc`.  The real one is 200-odd
  -- files that change whenever a doc comment does, so a golden over it
  -- would re-baseline on documentation edits while saying nothing more
  -- about helptags_one; the fixture is three files this sweep owns, and
  -- its tags file is generated by the command under test.
  local HELPRT = 'files/hrt'
  local HELPSETUP = table.concat({
    'call mkdir("' .. HELPRT .. '/doc", "p")',
    'call writefile(['
      .. '"*csa.txt*\tFirst fixture help file",'
      .. '"",'
      .. '"ALPHA topic\t\t\t\t*cs-alpha* *cs-first*",'
      .. '"A line that mentions needle once.",'
      .. '"Another line, no match here.",'
      .. '"needle again, twice: needle.",'
      .. '"|cs-beta| is over in the other file.",'
      .. '"vim:tw=78:ts=8:ft=help:norl:"'
      .. '], "' .. HELPRT .. '/doc/csa.txt")',
    'call writefile(['
      .. '"*csb.txt*\tSecond fixture help file",'
      .. '"",'
      .. '"BETA topic\t\t\t\t*cs-beta*",'
      .. '"no match on this line at all",'
      .. '"a NEEDLE in capitals",'
      .. '"vim:tw=78:ts=8:ft=help:norl:"'
      .. '], "' .. HELPRT .. '/doc/csb.txt")',
    'call writefile(["not a help file"], "' .. HELPRT .. '/doc/csc.notxt")',
    'let &runtimepath = "' .. HELPRT .. '"',
  }, ' | ')
  -- The quickfix list is the answer, and its `bufnr` is a handle whose
  -- value depends on how many buffers every case above happened to open --
  -- so the readback names the buffer, its line, its column and its text,
  -- and never the handle.
  local QF =
    'string(map(getqflist(), {_, v -> [fnamemodify(bufname(v.bufnr), ":t"), v.lnum, v.col, v.valid, v.text]}))'
  for _, line in ipairs({
    'helptags ' .. HELPRT .. '/doc',
    'helptags! ' .. HELPRT .. '/doc',
    'helptags ALL',
    'helptags files/nosuchdir',
    'helpgrep needle',
    'helpgrep NEEDLE',
    'helpgrep \\cneedle',
    'helpgrep needle\\|BETA',
    'helpgrep nomatchanywhere',
    'helpgrep',
    'helpgrep \\(',
    'helpgrep ^A',
    'lhelpgrep needle',
    'helpgrep! needle',
    '2helpgrep needle',
  }) do
    exec('c17g/' .. tag(line), line, {
      pre = HELPSETUP .. ' | silent! helptags ' .. HELPRT .. '/doc',
      after = QF,
    })
  end
  -- The list, walked: :helpgrep leaves a quickfix list behind and the
  -- commands that read it are the other half of the contract.
  for _, line in ipairs({ 'clist', 'cfirst', 'cnext', 'clast', 'cc 2', 'copen' }) do
    exec('c17q/' .. tag(line), line, {
      pre = HELPSETUP
        .. ' | silent! helptags '
        .. HELPRT
        .. '/doc | silent! helpgrep needle',
      after = 'string([getqflist({"idx": 0}), fnamemodify(bufname("%"), ":t"), &buftype])',
    })
  end
end)

-- ---------------------------------------------------------------------
-- s18 -- 'inccommand', and ex_substitute_preview behind it.
-- ---------------------------------------------------------------------

-- The preview callback runs only from the COMMAND LINE: `command_line_changed`
-- asks for it when 'inccommand' is set, the first character is `:`, nothing is
-- sourcing, and the typeahead is empty.  nvim_cmd never goes near it, which is
-- why s10 cannot reach it and why this section drives a CHILD over RPC
-- instead: `nvim_input` the command line, then read the answer back.
--
-- What is readable is the crux.  `cmdpreview_may_show` UNDOES the preview
-- before it returns -- `cmdpreview_restore_state` is the last thing it does --
-- so the buffer is never observably modified and the highlights live in an
-- ANONYMOUS namespace that `nvim_get_namespaces()` does not list.  The one
-- durable artifact is the `[Preview]` BUFFER that 'inccommand'=split opens:
-- `cmdpreview_close_win` closes the window and leaves the buffer, holding
-- exactly what ex_substitute_preview wrote into it -- `|N| <the substituted
-- line>` per match.  That buffer is this section's answer.
--
-- Synchronisation is content-based, not a sleep: poll `getcmdline()` until it
-- equals the text that was sent.  An RPC request is served when the child
-- blocks for input, which is after `command_line_changed` (and therefore
-- after the preview) has returned -- so a matching command line proves the
-- preview for that keystroke is already done.

section('s18-preview', function()
  local ok0, chan = pcall(vim.fn.jobstart, {
    vim.v.progpath, '--embed', '-u', 'NONE', '-i', 'NONE',
  }, { rpc = true, on_stderr = function() end })
  if not ok0 or type(chan) ~= 'number' or chan <= 0 then
    emit('c18/spawn', 'A', 'CHILD-SPAWN-FAILED ' .. esc(tostring(chan)))
    return
  end

  local function req(...)
    local rok, res = pcall(vim.rpcrequest, chan, ...)
    return rok, res
  end

  local aok, aerr = req('nvim_ui_attach', 80, 24, {})
  emit('c18/attach', 'A', aok and 'ok' or esc(errtext(aerr)))
  if not aok then
    pcall(vim.fn.jobstop, chan)
    return
  end
  req(
    'nvim_command',
    'set report=99999 nomore belloff=all noswapfile shortmess+=F laststatus=0'
  )

  local PFIX = {
    'alpha one', 'alpha two alpha', 'beta three', 'alpha four', 'gamma five',
  }

  local function readpreview()
    local rok, res = req(
      'nvim_exec_lua',
      [[
        local out = {}
        for _, b in ipairs(vim.api.nvim_list_bufs()) do
          local n = vim.api.nvim_buf_get_name(b)
          if n:match('%[Preview%]$') then
            out[#out + 1] = 'name=' .. n
            out[#out + 1] = 'body=[' ..
              table.concat(vim.api.nvim_buf_get_lines(b, 0, -1, false), ' / ') .. ']'
          end
        end
        out[#out + 1] = 'buf=[' ..
          table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, false), ' / ') .. ']'
        out[#out + 1] = 'mode=' .. vim.api.nvim_get_mode().mode
        out[#out + 1] = 'wins=' .. #vim.api.nvim_tabpage_list_wins(0)
        -- 'inccommand'=nosplit opens no preview BUFFER, so the buffer's
        -- text and the window count say nothing about whether the preview
        -- ran.  Its changedtick does: the preview applies the substitution
        -- and then undoes it, and an undo is itself a change.
        out[#out + 1] = 'tick=' .. vim.api.nvim_buf_get_var(0, 'changedtick')
        return table.concat(out, ' | ')
      ]],
      {}
    )
    return rok and tostring(res) or ('RPC-ERR ' .. errtext(res))
  end

  local CASES = {
    '%s/alpha/XX',
    '%s/alpha/XX/g',
    '%s/alpha/XX/gc',
    '%s/alpha/',
    '%s/alpha',
    '%s/al',
    '2,4s/alpha/Q',
    '.,+2s/alpha/Q',
    '%s/nomatch/Y',
    '%s/alpha/\\=submatch(0) . "!"',
    '%s/alpha/\\=no_such_fn()',
    '%s/\\(al\\)\\(pha\\)/\\2\\1/g',
    '%s/alpha\\nbeta/J',
    '%s/\\(/X',
    '%s#alpha#H#',
    '%smagic/a.pha/M/',
    '%snomagic/a.pha/M/',
    '4,2s/alpha/R',
    '%s/alpha/XX/&',
    'g/alpha/s//G/',
  }

  for _, icm in ipairs({ 'nosplit', 'split', '' }) do
    local sok = req('nvim_command', 'set inccommand=' .. icm)
    emit('c18/set/' .. (icm == '' and 'off' or icm), 'A', tostring(sok))
    for _, line in ipairs(CASES) do
      local label = 'c18/' .. (icm == '' and 'off' or icm) .. '/' .. tag(line)
      label_once(label)
      req('nvim_command', 'silent! %bwipeout!')
      req('nvim_buf_set_lines', 0, 0, -1, false, PFIX)
      req('nvim_input', ':' .. line)
      local settled = vim.wait(4000, function()
        local rok, cl = req('nvim_eval', 'getcmdline()')
        return rok and cl == line
      end, 5)
      emit(label, 'A', (settled and '' or 'NOT-SETTLED ') .. esc(scrub(readpreview())))
      req('nvim_input', '<Esc>')
      vim.wait(2000, function()
        local rok, m = req('nvim_eval', 'mode()')
        return rok and m == 'n'
      end, 5)
      local rok2, after = req(
        'nvim_exec_lua',
        'return table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, false), " / ")',
        {}
      )
      emit(label, 'B', esc(scrub(rok2 and tostring(after) or errtext(after))))
    end
  end

  -- The command line ACCEPTED, not abandoned: the preview is undone and the
  -- real `:s` runs, and the two had better agree.
  req('nvim_command', 'set inccommand=split')
  for _, line in ipairs({ '%s/alpha/XX/g', '2,4s/alpha/Q', '%s/nomatch/Y' }) do
    local label = 'c18/accept/' .. tag(line)
    label_once(label)
    req('nvim_command', 'silent! %bwipeout!')
    req('nvim_buf_set_lines', 0, 0, -1, false, PFIX)
    req('nvim_input', ':' .. line)
    vim.wait(4000, function()
      local rok, cl = req('nvim_eval', 'getcmdline()')
      return rok and cl == line
    end, 5)
    req('nvim_input', '<CR>')
    vim.wait(2000, function()
      local rok, m = req('nvim_eval', 'mode()')
      return rok and m == 'n'
    end, 5)
    local rok, after = req(
      'nvim_exec_lua',
      'return table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, false), " / ")',
      {}
    )
    emit(label, 'A', esc(scrub(rok and tostring(after) or errtext(after))))
  end

  -- REAP it, do not merely signal it.  `jobstop` returns as soon as the
  -- signal is sent, and an `--embed` child that outlives the sweep still
  -- holds $WORK as its cwd -- so the NEXT run's `rm -rf $WORK` races it.
  -- One mutation run in eight failed that way before this line existed.
  pcall(vim.fn.jobstop, chan)
  pcall(vim.fn.jobwait, { chan }, 5000)
end)

-- ---------------------------------------------------------------------
-- s20 -- the uncaptured message path.
-- ---------------------------------------------------------------------

section('s20-messages', function()
  -- 'report' at 0 so that every line-count message is emitted, and
  -- nothing captured: these go to the prompt, which in a headless
  -- process is the .stderr artifact -- the only view of msg_* here.
  local CASES = {
    '1,5delete',
    '1,5yank',
    '1,5copy 0',
    '1,5move $',
    '1,5join',
    '1,5>',
    '%s/alpha/A/g',
    '%s/alpha/A/gn',
    '%s/nomatchxyz/A/',
    '1,3print',
    '1,3number',
    '1,3list',
    'ls',
    'files',
    'buffers',
    'set nu?',
    'set all&',
    'command',
    'digraph a: 228',
    'history :',
    'args',
    -- NOT `:version`: its output names the build (`dev-<rev>`) and the
    -- feature list, so it identifies the binary rather than its
    -- behaviour, and would be a permanent expected-diff on both the
    -- baseline and the paired comparison.
    'Qwerty',
    '1,3Qwerty',
    'echoerr "boom"',
    'echomsg "note"',
    'echo "plain"',
    'echon "no-nl"',
    'let g:cs_m = 1',
    'unlet g:cs_nosuch',
    'call NoSuchFn()',
    'sort zz',
    'normal! 99G',
    'undo',
    'redo',
    'breaklist',
    'scriptnames',
  }
  for _, line in ipairs(CASES) do
    reset(nil, { 4, 0 }, 'report=0')
    io.stderr:write('### ', line, '\n')
    local pok, dict = pcall(vim.api.nvim_parse_cmd, line, {})
    if pok then
      if dict.reg == '' then
        dict.reg = nil
      end
      pcall(vim.api.nvim_cmd, dict, { output = false })
    else
      io.stderr:write('parse: ', errtext(dict), '\n')
    end
  end
  emit('s20', 'cases', tostring(#CASES))
end)

-- ---------------------------------------------------------------------
-- s91 -- CRASHPROBE.
-- ---------------------------------------------------------------------

-- Inputs whose failure mode is "the editor stops existing" rather than
-- "an error is reported": deep nesting, unbounded counts, addresses that
-- overflow, and the shapes that P0.3 turned from aborts into saturating
-- answers.  Each runs in a child, so a crash is one diffable row.
local CRASH = {}
do
  local LINES = {
    '9223372036854775807print',
    '-9223372036854775808print',
    '99999999999999999999print',
    '1,9223372036854775807print',
    '9223372036854775807,9223372036854775807print',
    '.+9223372036854775807print',
    '$-9223372036854775807print',
    '2147483647tab split',
    '2147483648tab split',
    '9999999999tab split',
    '2147483647verbose set',
    '2147483648verbose set',
    'buffer 9223372036854775807',
    'z 9223372036854775807',
    'command! -count=2147483648 CsX echo 1',
    'command! -nargs=2147483648 CsX echo 1',
    'command! -range=2147483648 CsX echo 1',
    'set cinoptions=>2147483648',
    'set breakindentopt=min:99999999999999999999999',
    'set rulerformat=%2147483647(x%)',
    'call search("x", "", 0, 99999999999999)',
    'digraph a: 2147483648',
    'digraph a: 99999999999999999999',
    'history : 2147483648',
    'history : -2147483649',
    '1,2147483648sort',
    'argdelete 2147483648',
    'z=2147483648',
    'z 2147483648',
    string.rep('silent ', 200) .. 'echo 1',
    string.rep('vertical ', 200) .. 'split',
    string.rep(':', 500) .. 'echo 1',
  }
  for _, line in ipairs(LINES) do
    CRASH[#CRASH + 1] = { 'k91/' .. tag(line), line }
  end
  -- The repetition cases get a *named* label: tag() of a 2,000-character
  -- line is a 2,000-character label, and a report line that long is
  -- unreadable in a diff and unusable in a mutation note.
  for _, spec in ipairs({
    { 'rep-silent-200', string.rep('silent ', 200) .. 'echo 1' },
    { 'rep-vertical-200', string.rep('vertical ', 200) .. 'split' },
    { 'rep-colon-500', string.rep(':', 500) .. 'echo 1' },
    { 'rep-range-100', string.rep('1,', 100) .. '2print' },
    { 'rep-paren-200', 'echo ' .. string.rep('(', 200) .. '1' .. string.rep(')', 200) },
    { 'rep-regexgroup-60', 'g/' .. string.rep('\\(', 60) .. 'x' .. string.rep('\\)', 60) .. '/d' },
    { 'rep-bar-200', 'echo 1' .. string.rep('|echo 1', 200) },
    { 'rep-plus-500', string.rep('+', 500) .. 'print' },
    { 'rep-mark-100', string.rep("'a+", 100) .. '0print' },
  }) do
    CRASH[#CRASH + 1] = { 'k91/' .. spec[1], spec[2] }
  end
  -- Hand-built Dicts, which no command line can express.  They go into
  -- nvim_cmd directly (the `dict` slot), because every integer field is
  -- caller input that reaches an `int` -- and three of the four aborts
  -- this section found on its first run were exactly here.
  for _, spec in ipairs({
    { 'dict-tab-intmax', { cmd = 'echo', args = { '1' }, mods = { tab = 2147483647 } } },
    { 'dict-tab-i64max', { cmd = 'echo', args = { '1' }, mods = { tab = 9223372036854775807 } } },
    { 'dict-verbose-intmax', { cmd = 'echo', args = { '1' }, mods = { verbose = 2147483647 } } },
    { 'dict-verbose-i64max', { cmd = 'echo', args = { '1' }, mods = { verbose = 9223372036854775807 } } },
    { 'dict-range-i64max', { cmd = 'print', range = { 9223372036854775807 } } },
    { 'dict-range-i64min', { cmd = 'print', range = { -9223372036854775807 - 1 } } },
    { 'dict-count-i64max', { cmd = 'buffer', count = 9223372036854775807 } },
    { 'dict-count-i64min', { cmd = 'buffer', count = -9223372036854775807 - 1 } },
    { 'dict-args-many', { cmd = 'echo', args = (function()
      local a = {}
      for i = 1, 500 do
        a[i] = tostring(i)
      end
      return a
    end)() } },
    { 'dict-cmd-long', { cmd = string.rep('a', 4096) } },
    { 'dict-nextcmd-long', { cmd = 'echo', args = { '1' }, nextcmd = string.rep('echo 1|', 200) } },
  }) do
    CRASH[#CRASH + 1] = { 'k91/' .. spec[1], spec[2] }
  end
  -- B16-5: the runtime family's Ex-command surface.  `cmdsweep`'s s91 is
  -- one of only two things in the toolbox that can see an abort (the
  -- other is abortprobe.py), and before this it had nothing on
  -- :runtime / :source / :packadd / :scriptnames.
  for _, line in ipairs({
    'runtime START',
    'runtime! ' .. string.rep('*', 40),
    'packadd ' .. string.rep('x', 4096),
    'packadd ' .. string.rep('../', 100) .. 'etc',
    'source ' .. string.rep('x', 4096),
    '1,2147483648source',
    '9223372036854775807source',
    'scriptnames 2147483648',
    -- NOT `scriptnames -2147483649` (a Lua traceback whose path Lua
    -- has already ELIDED, so the driver's sed cannot scrub it and the
    -- row names the worktree) and NOT `2147483647scriptnames` (the
    -- whole runtime tree's script list, which re-baselines this sweep
    -- whenever a runtime/*.vim file changes).  Both are in rtsweep's
    -- s91 instead, where the environment is fixture-only.
    'scriptencoding ' .. string.rep('x', 4096),
    'call getscriptinfo({"sid": 9223372036854775807})',
    'call getscriptinfo({"name": "' .. string.rep('x', 2048) .. '"})',
    'call ' .. string.rep('a#', 400) .. 'f()',
    'let &runtimepath = repeat("x", 65536)',
    'let &packpath = repeat(",", 20000)',
  }) do
    CRASH[#CRASH + 1] = { 'k91/' .. tag(line), line }
  end
  -- B17-5: the three B17 families whose extreme inputs nothing looked at.
  -- ex_session builds a path out of the buffer name and 'viewdir' by hand
  -- (get_view_file xmallocs len + 9), help.rs walks every doc/*.txt in
  -- 'runtimepath', and digraph.rs parses a decimal by hand.
  for _, line in ipairs({
    'mksession ' .. string.rep('x', 4096),
    'mkview 2147483648',
    'mkview 99999999999999999999',
    'let &viewdir = repeat("x", 8192) | mkview',
    'set viewdir= | mkview',
    'loadview 2147483648',
    'source ' .. string.rep('=', 4096),
    'helpgrep ' .. string.rep('x', 4096),
    'helpgrep \\%[' .. string.rep('a', 2048),
    'helptags ' .. string.rep('x', 4096),
    '2147483648helpgrep x',
    'help ' .. string.rep('x', 4096),
    'digraph ' .. string.rep('a', 4096) .. ' 65',
    'digraph a: -9223372036854775808',
    'digraph a: 9223372036854775807',
    'digraph a: 99999999999999999999',
    'digraphs ' .. string.rep('a: ', 2000),
    'history : 9223372036854775807',
    'history all 9223372036854775807,9223372036854775807',
  }) do
    CRASH[#CRASH + 1] = { 'k91/' .. tag(line), line }
  end
  -- Parse-only, and this is why.  A huge argument to these two is not a
  -- crash, it is a *wedge*: `:sleep` with 2^63-1 milliseconds sleeps for
  -- three hundred million years, and `:normal` with that count runs the
  -- key that many times.  A child that hangs is reported as ABORTED
  -- exactly like one that died, so it would be a false positive forever.
  for _, line in ipairs({
    'sleep 9223372036854775807',
    'sleep 99999999999999999999',
    'normal 9223372036854775807x',
    '9223372036854775807normal x',
  }) do
    CRASH[#CRASH + 1] = { 'k91/' .. tag(line), line, true }
  end
end

--- One crash case, printed unbuffered as `<idx> <label> <answer>`.
--- Shared by parent and child so the two agree on the spelling.
local function crashline(i)
  local c = CRASH[i]
  if type(c[2]) == 'table' then
    local xok, xres = pcall(vim.api.nvim_cmd, c[2], { output = true })
    io.write(
      i,
      ' ',
      c[1],
      ' X ',
      esc(scrub(xok and ('= ' .. tostring(xres)) or ('! ' .. errtext(xres)))),
      '\n'
    )
    return
  end
  local ok, res = pcall(vim.api.nvim_parse_cmd, c[2], {})
  local pout = ok and flat(res) or ('! ' .. errtext(res))
  io.write(i, ' ', c[1], ' P ', esc(scrub(pout)), '\n')
  local xout
  if c[3] then
    xout = '(parse-only)'
  elseif ok then
    local xok, xres = pcall(vim.api.nvim_cmd, res, { output = true })
    xout = xok and ('= ' .. tostring(xres)) or ('! ' .. errtext(xres))
  else
    xout = '(unparsed)'
  end
  io.write(i, ' ', c[1], ' X ', esc(scrub(xout)), '\n')
end

section('s91-crashprobe', function()
  local progpath = vim.v.progpath
  local i, guard = 1, 0
  while i <= #CRASH and guard < #CRASH + 40 do
    guard = guard + 1
    local res = vim
      .system({
        progpath,
        '--headless',
        '-u',
        'NONE',
        '-i',
        'NONE',
        '-l',
        script,
        '--child',
        tostring(i),
        -- A safety net, not the assertion: every input here is meant to
        -- finish or die.  A child that hits the timeout is killed and
        -- reads as ABORTED, so anything that *wedges* belongs on the
        -- parse-only list above instead.
      }, { text = true, cwd = work, timeout = 60000 })
      :wait()
    local last = i - 1
    for line in (res.stdout or ''):gmatch('[^\n]+') do
      local idx, kind = line:match('^(%d+) %S+ (%u) ')
      idx = tonumber(idx)
      if idx then
        -- A case prints TWO lines under one index, `P` then `X`, and it
        -- is the `X` that runs the command.  Advancing `last` on the `P`
        -- line blames the *next* case for a death during execution and
        -- silently skips it -- which is what happened the first time a
        -- pre-fix binary was run through this section: nine aborts, all
        -- attributed one case late.
        if kind == 'X' then
          last = idx
        end
        emit((line:gsub('^%d+ ', '')))
      end
    end
    -- vim.system reports an abort as a SIGNAL and leaves `code` at 0,
    -- so a code-only check calls a SIGABRT a success.
    local died = (res.signal or 0) ~= 0 or (res.code or 0) ~= 0
    if not died and last >= #CRASH then
      break
    end
    local dead = last + 1
    if dead <= #CRASH then
      emit(CRASH[dead][1], 'ABORTED')
      struct(CRASH[dead][1], { aborted = true })
    end
    i = dead + 1
  end
  emit('k91 groups', tostring(guard))
end)

-- ---------------------------------------------------------------------
-- Run.
-- ---------------------------------------------------------------------

-- Every option any section reads is set explicitly: a sweep that
-- inherits one is a sweep whose baseline moves when a default does.
quiet('set noswapfile nomore noshowmode shortmess=filnxtToOFS report=9999 belloff=all')
quiet('set encoding=utf-8 fileencoding= ambiwidth=single')
quiet('set columns=80 lines=24 cmdheight=1 laststatus=0 ruler& showcmd&')
quiet('set shell=/bin/sh shellxquote= noshellslash shellcmdflag=-c')
quiet('set nowritebackup nobackup hidden undofile& undolevels=1000')
quiet('language C')
quiet(DEFAULTS)

-- A FIXTURE clipboard provider, verbatim from opsweep: `"+` and `"*` go
-- through provider#clipboard#Call, and without this a `:put +` case
-- reads the host's real clipboard (or prints "No provider", which is a
-- function of $DISPLAY).  g:cbstore is the whole "system" clipboard for
-- this run, and cache_enabled is 0 so a paste always calls back in.
quiet([[
  let g:cbstore = {'+': [['CLIP'], 'v'], '*': [['CLIP'], 'v']}
  function! CsCbCopy(reg, lines, regtype) abort
    let g:cbstore[a:reg] = [a:lines, a:regtype]
  endfunction
  function! CsCbPaste(reg) abort
    return get(g:cbstore, a:reg, [[''], 'v'])
  endfunction
  let g:clipboard = {
        \ 'name': 'cmdsweep-fixture',
        \ 'copy': {
        \   '+': {lines, regtype -> CsCbCopy('+', lines, regtype)},
        \   '*': {lines, regtype -> CsCbCopy('*', lines, regtype)},
        \ },
        \ 'paste': {
        \   '+': {-> CsCbPaste('+')},
        \   '*': {-> CsCbPaste('*')},
        \ },
        \ 'cache_enabled': 0,
        \ }
]])

-- The scratch buffer.  One for the whole run: a fresh buffer per case
-- would hand out thousands of monotonic handles and make the artifact a
-- function of where in the run a case sits.
quiet('enew!')
BUF = vim.api.nvim_get_current_buf()
quiet('setlocal buftype=nofile bufhidden=hide noswapfile')

-- The child branch sits *below* the option block, so a child's answers
-- are computed under the same options as the parent's -- several of the
-- crash cases parse an address, which reads 'wrapscan' and 'magic'.
if child_from then
  for i = child_from, #CRASH do
    crashline(i)
  end
  os.exit(0)
end

-- Captured before any section runs, and before s09 defines or clears
-- anything: the binary's own command table plus whatever the runtime
-- installed.
ALLCMDS = vim.fn.getcompletion('', 'command')
table.sort(ALLCMDS)

emit('cwd', esc(scrub(vim.fn.getcwd())))
emit('runtime-tags', vim.fn.filereadable(runtime .. '/doc/tags'))
emit('encoding', vim.o.encoding)
emit('commands', tostring(#ALLCMDS))

for _, entry in ipairs(SECTIONS) do
  if not only or entry.name:match(only) then
    local started = trace and vim.uv.hrtime() or 0
    if trace then
      io.stderr:write('== ', entry.name, '\n')
    end
    emit('')
    emit('== ' .. entry.name .. ' ==')
    local ok, err = pcall(entry.fn)
    if not ok then
      emit('== ' .. entry.name .. ' ABORTED:', esc(errtext(err)))
    end
    if trace then
      io.stderr:write(string.format('   %s %.1fs\n', entry.name, (vim.uv.hrtime() - started) / 1e9))
    end
  end
end

emit('')
emit('== done ==')
if structfd then
  structfd:close()
end
