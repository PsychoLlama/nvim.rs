-- rtsweep -- the sixteenth differential.  Driven by rtsweep.sh,
-- which builds the fixture tree, fixes the environment and does the two
-- scrubs that only the shell can see.  Read that header first.
--
-- The subsystem is runtime/{estack,search,cache,pack,expand,rtp,source,
-- script}.rs.  Its characteristic regression is not an error, it is a
-- WRONG WINNER: the second candidate on 'runtimepath' sourced instead of
-- the first, an `after/` directory ordered before the tree it comes
-- after, a packadd'd directory inserted at the wrong index.  So the unit
-- of measurement here is an ORDERED LIST -- every fixture script appends
-- its own path to `g:RTM`, and the answer to "what did :runtime do" is
-- that list, in order, not "did it succeed".
--
-- Sections:
--   s01 rtp        rtp.rs -- 'runtimepath'/'packpath' as CONSTRUCTED, one
--                  child process per environment (it is built before any
--                  script of ours could run)
--   s02 runtime    search.rs -- :runtime / :runtime! x START/OPT/PACK/ALL
--   s03 search     cache.rs -- the cached search path as a golden
--                  (nvim__runtime_inspect) + nvim_get_runtime_file
--   s04 pack       pack.rs -- :packadd[!], :packloadall[!], and the
--                  'runtimepath' each one leaves behind
--   s05 expand     expand.rs -- ExpandRTDir / ExpandPackAddDir /
--                  expand_runtime_cmd through getcompletion()
--   s06 source     source.rs -- :source of file / buffer / range / Lua,
--                  :finish, :scriptencoding, continuations, heredocs
--   s07 estack     estack.rs -- <sfile> <slnum> <sflnum> <script> <stack>
--                  and getstacktrace(), nested
--   s08 script     script.rs -- :scriptnames, getscriptinfo(), autoload
--   s20 messages   the uncaptured block that fills .stderr
--   s91 crashprobe the inputs that may kill the editor, one child each
--
-- Everything printed has to be reproducible across two builds run
-- minutes apart and from two working directories, so the report carries
-- no address, pid, wall-clock time or path outside the work directory.
-- Script IDs *are* printed: they are a deterministic function of the
-- section order, and a rewrite that renumbers them has changed something
-- real.  That is also why RTSWEEP_ONLY is for iterating, not for gating.

local work = assert(os.getenv('RT_WORK'), 'RT_WORK unset')
local runtime = os.getenv('VIMRUNTIME') or ''
local script = debug.getinfo(1, 'S').source:sub(2)

local argv = _G.arg or {}
local child_mode = argv[1]

local only = os.getenv('RTSWEEP_ONLY')
if only == '' then
  only = nil
end
local trace = os.getenv('RTSWEEP_TRACE') == '1'

io.stdout:setvbuf(child_mode and 'no' or 'line')

local function emit(...)
  io.write(table.concat({ ... }, ' '), '\n')
end

-- --------------------------------------------------------------- scrub

--- Strip the bits of an answer that name where -- or when -- the run
--- happened.  Sorting happens after this, never before.
local function scrub(text)
  text = tostring(text)
  text = text:gsub(vim.pesc(work), '<WORK>'):gsub(vim.pesc(script), '<SCRIPT>'):gsub('%.%.%.[^%s\'"]-rtsweep%.lua', '<SCRIPT>')  -- $HERE before $VIM (below), and truncation-proof: LuaJIT elides a chunk name past ~60 chars, and since the harness moved into the checkout $VIM is a PREFIX of the script path
  if runtime ~= '' then
    text = text:gsub(vim.pesc(runtime), '<RT>')
    -- `$VIM` is VIMRUNTIME with its last component removed, so it names
    -- the checkout this binary was built from.  Two worktrees of the
    -- same revision would differ here and nowhere else.
    text = text:gsub(vim.pesc((runtime:gsub('/[^/]+$', ''))), '<VIMDIR>')
  end
  -- (the <SCRIPT> mask used to sit here; it now runs first, see above)
  -- Not `[^%s'"]*`: a `:scriptnames` row is "  7: /path", and a greedy
  -- class that may cross a colon eats the script ID with the path.
  text = text:gsub('[^%s\'":]*/vim/_core/', '<CORE>/')
  text = text:gsub('/tmp/nvim%.[%w_%-%.]+', '<NVIMTMP>')
  -- The baked dependency prefix carries a store hash, which is a
  -- function of the build inputs and not of this subsystem.
  text = text:gsub('/nix/store/%w+%-', '<STORE>/')
  text = text:gsub('%d+ ' .. '(second)s? ago', 'N %1s ago')
  -- s91 feeds 4k-character names in and gets them back inside an error
  -- message.  The run LENGTH is the information; the run is not.  Lua
  -- patterns cannot quantify a backreference, so this is a hand loop.
  if #text >= 40 then
    local out, i, n = {}, 1, #text
    while i <= n do
      local c = text:sub(i, i)
      local j = i
      while j < n and text:sub(j + 1, j + 1) == c do
        j = j + 1
      end
      out[#out + 1] = (j - i + 1 >= 20) and string.format('%s<x%d>', c, j - i + 1)
        or text:sub(i, j)
      i = j + 1
    end
    text = table.concat(out)
  end
  return text
end

--- Cap a message.  Errors are where this corpus' bloat lives (a
--- recursive :source names its whole 200-frame stack), and a capped
--- length is as diffable as an uncapped one.
local function cap(text, limit)
  limit = limit or 240
  if #text <= limit then
    return text
  end
  return text:sub(1, limit) .. string.format('...<+%d>', #text - limit)
end

--- Escape to one printable line: a byte difference has to show in the
--- diff, and a report line has to stay a report line.
local function esc(bytes)
  return (tostring(bytes):gsub('[%c\128-\255\\]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

--- A case label is a name, not the input: this corpus' inputs are paths
--- and a path-shaped label would be scrubbed along with everything else.
--- The DUPLICATE-LABEL check below is the standing proof that the names
--- stayed injective.
local SEEN = {}
local function label_once(label)
  if SEEN[label] then
    emit('!! DUPLICATE LABEL', label)
  end
  SEEN[label] = true
  return label
end

-- -------------------------------------------------------------- struct

local structfd
if not child_mode then
  structfd = assert(io.open(assert(os.getenv('RT_STRUCT'), 'RT_STRUCT unset'), 'w'))
end

--- One-line quoting.  `string.format('%q', s)` writes a real newline for
--- \n in LuaJIT, which would split a struct row in two.
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
  if structfd then
    structfd:write(label, '\t', canon(value), '\n')
  end
end

--- Normalise an error to its message.  A pcall against vim.cmd prefixes
--- the Lua source position, which is a line number in *this* file and
--- would re-baseline the whole artifact on any edit above it.
local function errtext(res)
  local s = tostring(res)
  -- A Lua traceback names line numbers in THIS file and in the runtime's
  -- own _editor.lua.  Keeping either would re-baseline the artifact on
  -- an edit anywhere above the case, which is not a behaviour change.
  s = s:gsub('\nstack traceback:.*$', '')
  s = s:gsub('^[^\n]-rtsweep%.lua:%d+: ', '')
  s = s:gsub('^%[string "[^"]*"%]:%d+: ', '')
  s = s:gsub('\r?\n', ' | ')
  return cap(scrub(s))
end

-- ------------------------------------------------------------ sections

local function section(name, fn)
  if only and not name:match(only) then
    return
  end
  if trace then
    io.stderr:write('== ' .. name .. '\n')
  end
  emit('##', name)
  local ok, err = pcall(fn)
  if not ok then
    emit('##', name, 'RAISED', esc(scrub(errtext(err))))
  end
end

-- ------------------------------------------------------------- fixture

-- The rtp entries, in the order the search must honour.  d1/d2/d3 all
-- carry plugin/p1.vim, so "the first one" and "all of them" are
-- different answers; rt/xa/after is an rtp entry that IS an `after`
-- directory, which is the only thing `path_is_after` looks at.
local RTP = { 'rt/d1', 'rt/d2', 'rt/d3', 'rt/xa/after' }
local PACKP = { 'pk', 'pk2' }

local function abs(list)
  local out = {}
  for i, p in ipairs(list) do
    out[i] = work .. '/' .. p
  end
  return out
end

--- `raw` is a 'runtimepath' string used verbatim -- the only way to get
--- a RELATIVE entry into the option, which `abs()` would otherwise make
--- absolute and which is the only shape short enough to reach
--- `path_is_after`'s length guard.
local function reset(rtp, pp, raw)
  vim.cmd('let g:RTM = []')
  vim.o.runtimepath = raw or table.concat(abs(rtp or RTP), ',')
  vim.o.packpath = table.concat(abs(pp or PACKP), ',')
end

local function markers()
  local m = vim.g.RTM
  if type(m) ~= 'table' then
    return {}
  end
  return m
end

local function ins(value)
  return (vim.inspect(value, { newline = ' ', indent = '' }))
end

--- The marker list, which is the answer for most of this sweep.
local function report_markers(label)
  local m = markers()
  emit(label, 'M', esc(scrub(table.concat(m, ' '))))
  struct(label .. '#M', m)
end

--- A comma list ('runtimepath', a glob result) as an ordered R line.
local function report_list(label, list)
  emit(label, 'R', esc(scrub(table.concat(list, ' '))))
  struct(label .. '#R', list)
end

local function split_rtp(value)
  return vim.split(value, ',', { plain = true })
end

--- The case's own extra question, evaluated as Vimscript.
local function ask(label, expr)
  local ok, value = pcall(vim.api.nvim_eval, expr)
  emit(label, 'A', esc(scrub(ok and ins(value) or ('! ' .. errtext(value)))))
  struct(label .. '#A', ok and value or ('! ' .. errtext(value)))
end

--- One command case: reset the world, run the line, report the markers.
local function case(label, line, o)
  o = o or {}
  label_once(label)
  reset(o.rtp, o.pp, o.rtp_raw)
  if o.pre then
    pcall(vim.cmd, o.pre)
  end
  local ok, err = pcall(vim.cmd, line)
  if not ok then
    emit(label, '!', esc(scrub(errtext(err))))
    struct(label .. '#!', errtext(err))
  end
  report_markers(label)
  if o.rtp_after then
    report_list(label .. '/rtp', split_rtp(vim.o.runtimepath))
  end
  if o.ask then
    ask(label, o.ask)
  end
end

-- =====================================================================
-- Child modes.
-- =====================================================================

-- s01 asks what 'runtimepath' *is* under a given environment, and that
-- is decided before any script of ours could run: one process per
-- environment is the only way to see it.
if child_mode == '--rtp' then
  local function line(key, value)
    io.write(key, '\t', tostring(value), '\n')
  end
  line('rtp', vim.o.runtimepath)
  line('packpath', vim.o.packpath)
  line('$VIM', vim.env.VIM or '<unset>')
  line('$VIMRUNTIME', vim.env.VIMRUNTIME or '<unset>')
  for _, key in ipairs({
    'config',
    'data',
    'state',
    'cache',
    'log',
    'run',
    'config_dirs',
    'data_dirs',
  }) do
    local ok, value = pcall(vim.fn.stdpath, key)
    line('stdpath.' .. key, ok and ins(value) or ('! ' .. errtext(value)))
  end
  os.exit(0)
end

-- s91's corpus: inputs whose failure mode is "the editor stops existing"
-- rather than "an error is reported".  Each runs in a child, so a crash
-- is one diffable row.
local CRASH = {}
do
  local function add(name, line, opts)
    CRASH[#CRASH + 1] = { name, line, opts or {} }
  end
  add('rtp-10k-commas', 'let &runtimepath = repeat(",", 10000)')
  add('rtp-2k-entries', 'let &runtimepath = repeat("' .. work .. '/rt/d1,", 2000)')
  add('rtp-escaped-commas', 'let &runtimepath = repeat("a\\\\,", 2000)')
  add('rtp-one-huge-entry', 'let &runtimepath = repeat("x", 65536)')
  add('pp-10k-commas', 'let &packpath = repeat(",", 10000)')
  add('pp-2k-entries', 'let &packpath = repeat("' .. work .. '/pk,", 2000)')
  add('runtime-name-8k', 'runtime ' .. string.rep('x', 8192))
  add('runtime-bang-name-8k', 'runtime! ' .. string.rep('x', 8192))
  add('runtime-stars-40', 'runtime! ' .. string.rep('*', 40))
  add('runtime-deep-glob', 'runtime! ' .. string.rep('*/', 24) .. 'p1.vim')
  add('runtime-many-pats', 'runtime! ' .. string.rep('plugin/p1.vim ', 500))
  add('runtime-where-only', 'runtime START')
  add('runtime-empty', 'runtime')
  add('runtime-bang-empty', 'runtime!')
  add('packadd-name-4k', 'packadd ' .. string.rep('x', 4096))
  add('packadd-traversal', 'packadd ../../../etc')
  add('packadd-empty', 'packadd')
  add('packadd-slash', 'packadd a/b/c')
  add('packloadall-huge-pp', 'let &packpath = repeat("' .. work .. '/pk,", 500) | packloadall!')
  add('source-name-4k', 'source ' .. string.rep('x', 4096))
  add('source-devnull', 'source /dev/null')
  add('source-self-recursive', 'source ' .. work .. '/src/self.vim')
  add('source-range-i64max', '1,9223372036854775807source')
  add('source-range-zero', '0source')
  add('source-dir', 'source ' .. work .. '/src')
  add('scriptinfo-sid-i64max', 'call getscriptinfo({"sid": 9223372036854775807})')
  add('scriptinfo-sid-i64min', 'call getscriptinfo({"sid": -9223372036854775807-1})')
  add('scriptinfo-sid-neg', 'call getscriptinfo({"sid": -1})')
  add('scriptinfo-name-4k', 'call getscriptinfo({"name": "' .. string.rep('x', 4096) .. '"})')
  add('scriptnames-arg', 'scriptnames 9223372036854775807')
  add('autoload-500-deep', 'call ' .. string.rep('a#', 500) .. 'f()')
  add('autoload-name-8k', 'call ' .. string.rep('x', 8192) .. '#f()')
  add('finish-toplevel', 'finish')
  add('scriptencoding-bogus', 'scriptencoding nosuchencoding')
  add('scriptencoding-4k', 'scriptencoding ' .. string.rep('x', 4096))
  add('sfile-toplevel', 'let g:X = expand("<sfile>")')
  add('stack-toplevel', 'let g:X = expand("<stack>")')
  add('slnum-toplevel', 'let g:X = expand("<slnum>")')
  add('getstacktrace-toplevel', 'call getstacktrace()')
end

--- One crash case, printed unbuffered as `<idx> <label> X <answer>`.
--- Exactly ONE line per index, deliberately: cmdsweep's two-lines-per-
--- index shape made a death during execution blame the next case and
--- skip it, and a one-line case cannot.
local function crashline(i)
  local c = CRASH[i]
  reset()
  local ok, err = pcall(vim.cmd, c[2])
  io.write(i, ' k91/', c[1], ' X ', esc(scrub(ok and '= ok' or ('! ' .. errtext(err)))), '\n')
end

if child_mode == '--crash' then
  crashline(assert(tonumber(argv[2]), 'crash index'))
  os.exit(0)
end

-- =====================================================================
-- s01 -- 'runtimepath' and 'packpath' as constructed.
-- =====================================================================

local BASE_ENV = {
  PATH = '/usr/bin:/bin',
  HOME = work .. '/home',
  TERM = 'dumb',
  SHELL = '/bin/sh',
  LANG = 'C.UTF-8',
  VIMRUNTIME = runtime,
  XDG_CONFIG_HOME = work .. '/xdg/cfg1',
  XDG_DATA_HOME = work .. '/xdg/data1',
  XDG_STATE_HOME = work .. '/xdg/state1',
  XDG_CACHE_HOME = work .. '/xdg/cache1',
  XDG_RUNTIME_DIR = work .. '/xdg/run',
  XDG_CONFIG_DIRS = work .. '/xdg/cfg2',
  XDG_DATA_DIRS = work .. '/xdg/data2',
  NVIM_TEST = '1',
  -- The children re-run this same file, and it asserts on this.
  RT_WORK = work,
}

-- `false` means "unset this one", which is a different question from
-- "set it to the empty string" and both are in the corpus.
local RTP_CASES = {
  { 'base', {} },
  { 'no-config-home', { XDG_CONFIG_HOME = false } },
  { 'no-data-home', { XDG_DATA_HOME = false } },
  { 'no-state-home', { XDG_STATE_HOME = false } },
  {
    'no-xdg-at-all',
    {
      XDG_CONFIG_HOME = false,
      XDG_DATA_HOME = false,
      XDG_STATE_HOME = false,
      XDG_CACHE_HOME = false,
      XDG_CONFIG_DIRS = false,
      XDG_DATA_DIRS = false,
    },
  },
  { 'config-dirs-two', { XDG_CONFIG_DIRS = work .. '/xdg/cfg1:' .. work .. '/xdg/cfg2' } },
  { 'data-dirs-two', { XDG_DATA_DIRS = work .. '/xdg/data1:' .. work .. '/xdg/data2' } },
  { 'config-dirs-empty', { XDG_CONFIG_DIRS = '' } },
  { 'data-dirs-trailing', { XDG_DATA_DIRS = work .. '/xdg/data2:' } },
  { 'data-dirs-double-sep', { XDG_DATA_DIRS = work .. '/xdg/data1::' .. work .. '/xdg/data2' } },
  -- A comma in a directory name has to survive into a comma-separated
  -- option: `strcpy_comma_escaped` is the only thing that makes it.
  { 'comma-in-config', { XDG_CONFIG_HOME = work .. '/xdg/co,mma' } },
  { 'comma-in-data', { XDG_DATA_HOME = work .. '/xdg/co,mma' } },
  { 'comma-in-config-dirs', { XDG_CONFIG_DIRS = work .. '/xdg/co,mma' } },
  { 'relative-config', { XDG_CONFIG_HOME = 'relcfg' } },
  { 'empty-config-home', { XDG_CONFIG_HOME = '' } },
  { 'appname', { NVIM_APPNAME = 'myapp' } },
  { 'appname-nested', { NVIM_APPNAME = 'my/app' } },
  { 'appname-absolute', { NVIM_APPNAME = '/abs/bad' } },
  { 'appname-dotdot', { NVIM_APPNAME = '../escape' } },
  { 'vim-env-set', { VIM = work .. '/xdg/cfg1' } },
  { 'clean', {}, { '--clean' } },
}

section('s01-rtp', function()
  for _, spec in ipairs(RTP_CASES) do
    local label = label_once('k01/' .. spec[1])
    local env = {}
    for k, v in pairs(BASE_ENV) do
      env[k] = v
    end
    for k, v in pairs(spec[2]) do
      env[k] = (v ~= false) and v or nil
    end
    local args = { work .. '/bin/nvim', '--headless' }
    vim.list_extend(args, spec[3] or { '-u', 'NONE', '-i', 'NONE' })
    vim.list_extend(args, { '-l', script, '--rtp' })
    local res = vim
      .system(args, { text = true, cwd = work, env = env, clear_env = true, timeout = 60000 })
      :wait()
    emit(label, 'A', 'exit ' .. tostring(res.code) .. ' signal ' .. tostring(res.signal or 0))
    for out in (res.stdout or ''):gmatch('[^\n]+') do
      local key, value = out:match('^([^\t]*)\t(.*)$')
      if key == 'rtp' or key == 'packpath' then
        report_list(label .. '/' .. key, vim.split(value, ',', { plain = true }))
      elseif key then
        emit(label, 'A', key, esc(scrub(value)))
        struct(label .. '#' .. key, value)
      end
    end
    local err = (res.stderr or ''):gsub('%s+$', '')
    if err ~= '' then
      emit(label, '!', esc(scrub(err)))
    end
  end
end)

-- =====================================================================
-- s02 -- :runtime and :runtime!, and the four WHERE words.
-- =====================================================================

section('s02-runtime', function()
  for _, spec in ipairs({
    -- plain vs `!`: the first match, or every match in search order
    { 'plain-p1', 'runtime plugin/p1.vim' },
    { 'bang-p1', 'runtime! plugin/p1.vim' },
    { 'plain-glob', 'runtime plugin/p*.vim' },
    { 'bang-glob', 'runtime! plugin/p*.vim' },
    { 'bang-all-vim', 'runtime! plugin/*.vim' },
    { 'bang-all-lua', 'runtime! plugin/*.lua' },
    { 'plain-lua', 'runtime plugin/p1.lua' },
    { 'plain-lua-only', 'runtime plugin/p3.lua' },
    { 'two-pats', 'runtime plugin/p2.vim plugin/only3.vim' },
    { 'two-pats-bang', 'runtime! plugin/p1.vim plugin/only3.vim' },
    { 'two-pats-first-missing', 'runtime plugin/nope.vim plugin/only3.vim' },
    { 'nested-dir', 'runtime plugin/sub/deep.vim' },
    { 'starstar', 'runtime! **/deep.vim' },
    { 'colors-one', 'runtime colors/one.vim' },
    { 'colors-glob', 'runtime! colors/*.vim' },
    { 'syntax', 'runtime syntax/sx.vim' },
    { 'after-explicit', 'runtime! after/plugin/ap.vim' },
    { 'missing', 'runtime plugin/nope.vim' },
    { 'missing-bang', 'runtime! plugin/nope.vim' },
    -- the WHERE words.  START and OPT set DIP_NORTP, so they must NOT
    -- reach 'runtimepath'; PACK is both; ALL is both plus the rtp.
    { 'start-s1', 'runtime START plugin/s1.vim' },
    { 'start-bang', 'runtime! START plugin/*.vim' },
    { 'start-rtp-only-file', 'runtime START plugin/p1.vim' },
    { 'opt-o1', 'runtime OPT plugin/o1.vim' },
    { 'opt-bang', 'runtime! OPT plugin/*.vim' },
    { 'pack-bang', 'runtime! PACK plugin/*.vim' },
    { 'pack-plain', 'runtime PACK plugin/o1.vim' },
    { 'all-bang', 'runtime! ALL plugin/*.vim' },
    { 'all-plain', 'runtime ALL plugin/p1.vim' },
    { 'all-plain-opt', 'runtime ALL plugin/o1.vim' },
    { 'ftdetect-start', 'runtime! START ftdetect/*.vim' },
    -- the WHERE match is exact and case-sensitive, and a word that is
    -- not one of the four is a *pattern*, not an error
    { 'where-lowercase', 'runtime start plugin/s1.vim' },
    { 'where-prefix', 'runtime STAR plugin/s1.vim' },
    { 'where-suffix', 'runtime STARTX plugin/s1.vim' },
    { 'where-tab-sep', 'runtime\tSTART plugin/s1.vim' },
    { 'bang-where-all', 'runtime! ALL plugin/o1.vim' },
    -- a single-entry rtp, so "first match" has nothing to be confused
    -- with, and a reversed one, where the winner must change
    { 'single-entry', 'runtime plugin/p1.vim', { rtp = { 'rt/d2' } } },
    { 'reversed-rtp', 'runtime plugin/p1.vim', { rtp = { 'rt/d3', 'rt/d2', 'rt/d1' } } },
    { 'reversed-bang', 'runtime! plugin/p1.vim', { rtp = { 'rt/d3', 'rt/d2', 'rt/d1' } } },
    { 'empty-rtp', 'runtime! plugin/p1.vim', { rtp = { 'rt/nosuch' } } },
    { 'no-packpath', 'runtime! PACK plugin/*.vim', { pp = { 'pk/nosuch' } } },
    { 'one-packpath', 'runtime! PACK plugin/*.vim', { pp = { 'pk2' } } },
    -- A relative rtp entry spelled exactly `after`, which is the
    -- shortest string `path_is_after` can be asked about.
    { 'relative-after-entry', 'runtime! plugin/*.vim', { rtp_raw = work .. '/rt/d1,after' } },
    { 'relative-after-only', 'runtime! plugin/rel.vim', { rtp_raw = 'after' } },
    { 'relative-plain-entry', 'runtime! plugin/*.vim', { rtp_raw = 'rt/d2,rt/d3' } },
  }) do
    case('k02/' .. spec[1], spec[2], spec[3])
  end
end)

-- =====================================================================
-- s03 -- the cached search path, and the API's read-only views of it.
-- =====================================================================

--- nvim__runtime_inspect() with the paths made relative to $WORK, which
--- is what makes this a readable ordering golden rather than a wall of
--- absolute paths.
local function inspect_path(label)
  -- `nvim__runtime_inspect` reads the cached path WITHOUT validating it,
  -- and in `-l` script mode the event loop that would otherwise call
  -- `runtime_search_path_validate` never runs: without this line every
  -- case here answers with the snapshot taken at startup.  A search is
  -- what forces the rebuild (`runtime_search_path_get_cached`).
  pcall(vim.cmd, 'runtime zz_no_such_file_zz.vim')
  local items = vim.api.nvim__runtime_inspect()
  -- `pos_in_rtp` is a BYTE OFFSET into 'runtimepath', so its value is a
  -- function of how long $WORK's name is and two runs from two work
  -- directories disagree on every row.  Its meaning is entirely ordinal
  -- -- which rtp entry an inserted directory belongs to -- so report the
  -- rank among the distinct offsets, which carries exactly that and
  -- nothing else.
  local offsets, rank = {}, {}
  for _, item in ipairs(items) do
    offsets[item.pos_in_rtp or -1] = true
  end
  local sorted = {}
  for off in pairs(offsets) do
    sorted[#sorted + 1] = off
  end
  table.sort(sorted)
  for i, off in ipairs(sorted) do
    rank[off] = i - 1
  end
  local rows = {}
  for i, item in ipairs(items) do
    rows[i] = string.format(
      '%s[pos=%d%s%s%s]',
      (item.path or ''):gsub(vim.pesc(work) .. '/?', ''),
      rank[item.pos_in_rtp or -1],
      item.after and ' after' or '',
      item.pack_inserted and ' pack' or '',
      item.has_lua == nil and '' or (item.has_lua and ' lua' or ' nolua')
    )
  end
  emit(label, 'R', esc(scrub(table.concat(rows, ' '))))
  struct(label .. '#R', rows)
end

section('s03-search', function()
  reset()
  inspect_path(label_once('k03/inspect-base'))

  reset()
  vim.o.runtimepath = vim.o.runtimepath .. ',' .. work .. '/rt/d1/after'
  inspect_path(label_once('k03/inspect-after-appended'))

  reset()
  vim.o.runtimepath = work .. '/rt/d3,' .. work .. '/rt/d1'
  inspect_path(label_once('k03/inspect-two'))

  reset()
  vim.o.packpath = ''
  inspect_path(label_once('k03/inspect-no-packpath'))

  reset()
  pcall(vim.cmd, 'packadd o1')
  inspect_path(label_once('k03/inspect-after-packadd'))

  reset()
  vim.o.runtimepath = ''
  inspect_path(label_once('k03/inspect-empty-rtp'))

  reset(nil, nil, work .. '/rt/d1,after')
  inspect_path(label_once('k03/inspect-relative-after'))

  reset(nil, nil, 'rt/d2,rt/d3,after')
  inspect_path(label_once('k03/inspect-all-relative'))

  -- The invalidation itself: read the cache, change the option, read it
  -- again.  A cache that is not invalidated answers the first question
  -- twice, and every other section would still pass.
  reset()
  inspect_path(label_once('k03/invalidate-before'))
  vim.o.runtimepath = work .. '/rt/d2'
  inspect_path(label_once('k03/invalidate-after'))

  for _, spec in ipairs({
    { 'file-p1-first', { 'plugin/p1.vim' }, false },
    { 'file-p1-all', { 'plugin/p1.vim' }, true },
    { 'file-glob-first', { 'plugin/*.vim' }, false },
    { 'file-glob-all', { 'plugin/*.vim' }, true },
    { 'file-lua-all', { 'plugin/*.lua' }, true },
    { 'file-two-pats', { 'plugin/p2.vim', 'plugin/only3.vim' }, true },
    { 'file-missing', { 'plugin/nope.vim' }, true },
    { 'file-empty-pat', { '' }, true },
    { 'file-dir', { 'plugin' }, true },
    { 'file-dir-slash', { 'plugin/' }, true },
    { 'file-absolute', { work .. '/rt/d1/plugin/p1.vim' }, true },
    { 'file-parent', { '../d2/plugin/p1.vim' }, true },
    { 'file-after', { 'after/plugin/ap.vim' }, true },
    { 'file-starstar', { '**/deep.vim' }, true },
  }) do
    local label = label_once('k03/' .. spec[1])
    reset()
    local ok, res = pcall(vim.api.nvim_get_runtime_file, spec[2][1], spec[3])
    if #spec[2] > 1 then
      ok, res = pcall(function()
        local out = {}
        for _, pat in ipairs(spec[2]) do
          vim.list_extend(out, vim.api.nvim_get_runtime_file(pat, spec[3]))
        end
        return out
      end)
    end
    if not ok then
      emit(label, '!', esc(scrub(errtext(res))))
    else
      report_list(label, res)
    end
  end
end)

-- =====================================================================
-- s04 -- :packadd, :packloadall, and the 'runtimepath' they leave.
-- =====================================================================

--- 'runtimepath' as a set of *insertions*, relative to the fixture's
--- own list: which entries a pack command added and WHERE.
local function rtp_delta(label)
  local out = {}
  for i, entry in ipairs(split_rtp(vim.o.runtimepath)) do
    out[i] = (entry:gsub(vim.pesc(work) .. '/?', ''))
  end
  emit(label .. '/rtp', 'R', esc(scrub(table.concat(out, ' '))))
  struct(label .. '/rtp#R', out)
end

section('s04-pack', function()
  for _, spec in ipairs({
    { 'packadd-o1', 'packadd o1' },
    { 'packadd-bang-o1', 'packadd! o1' },
    { 'packadd-o2', 'packadd o2' },
    { 'packadd-o3-second-root', 'packadd o3' },
    { 'packadd-start-pkg', 'packadd s1' },
    { 'packadd-hollow', 'packadd hollow' },
    { 'packadd-missing', 'packadd nosuchpack' },
    { 'packadd-twice', 'packadd o1 | packadd o1' },
    { 'packadd-two', 'packadd o1 | packadd o2' },
    { 'packadd-bang-then-plain', 'packadd! o1 | packadd o1' },
    { 'packloadall', 'packloadall' },
    { 'packloadall-bang', 'packloadall!' },
    { 'packloadall-twice', 'packloadall | packloadall' },
    { 'packloadall-bang-twice', 'packloadall! | packloadall!' },
    { 'packadd-then-loadall', 'packadd o1 | packloadall' },
    { 'loadall-then-packadd', 'packloadall | packadd o1' },
    { 'packloadall-one-root', 'packloadall', { pp = { 'pk' } } },
    { 'packloadall-other-root', 'packloadall', { pp = { 'pk2' } } },
    { 'packloadall-no-packpath', 'packloadall', { pp = { 'pk/nosuch' } } },
    { 'packadd-empty-rtp', 'packadd o1', { rtp = { 'rt/d1' } } },
  }) do
    local o = spec[3] or {}
    case('k04/' .. spec[1], spec[2], o)
    rtp_delta('k04/' .. spec[1])
  end

  -- The start packages reach 'runtimepath' through the CACHE, not
  -- through an option write: `add_pack_start_dirs` runs on the search,
  -- so a bare :runtime is what makes them visible.
  local label = label_once('k04/start-dirs-via-search')
  reset()
  pcall(vim.cmd, 'runtime! plugin/s1.vim')
  report_markers(label)
  rtp_delta(label)
end)

-- =====================================================================
-- s05 -- ExpandRTDir / ExpandPackAddDir / expand_runtime_cmd.
-- =====================================================================

section('s05-expand', function()
  for _, spec in ipairs({
    { 'runtime-empty', '', 'runtime' },
    { 'runtime-plugin', 'plugin/', 'runtime' },
    { 'runtime-plugin-p', 'plugin/p', 'runtime' },
    { 'runtime-col', 'col', 'runtime' },
    { 'runtime-nomatch', 'zzz', 'runtime' },
    { 'runtime-star', '*', 'runtime' },
    { 'runtime-deep', 'plugin/sub/', 'runtime' },
    { 'packadd-empty', '', 'packadd' },
    { 'packadd-o', 'o', 'packadd' },
    { 'packadd-s', 's', 'packadd' },
    { 'packadd-nomatch', 'zzz', 'packadd' },
    { 'color-empty', '', 'color' },
    { 'color-o', 'o', 'color' },
  }) do
    local label = label_once('k05/' .. spec[1])
    reset()
    local ok, res = pcall(vim.fn.getcompletion, spec[2], spec[3])
    if ok then
      report_list(label, res)
    else
      emit(label, '!', esc(scrub(errtext(res))))
    end
  end

  -- The cmdline context walker (`set_context_in_runtime_cmd`): the WHERE
  -- word is completed too, and it changes what the rest completes over.
  for _, spec in ipairs({
    { 'cmd-runtime-bare', 'runtime ' },
    { 'cmd-runtime-partial', 'runtime pl' },
    { 'cmd-runtime-S', 'runtime S' },
    { 'cmd-runtime-START', 'runtime START ' },
    { 'cmd-runtime-START-plugin', 'runtime START plugin/' },
    { 'cmd-runtime-OPT', 'runtime OPT plugin/' },
    { 'cmd-runtime-PACK', 'runtime PACK plugin/' },
    { 'cmd-runtime-ALL', 'runtime ALL plugin/' },
    { 'cmd-runtime-bang', 'runtime! plugin/' },
    { 'cmd-runtime-two', 'runtime plugin/p1.vim plugin/' },
    { 'cmd-packadd', 'packadd ' },
    { 'cmd-packadd-o', 'packadd o' },
    { 'cmd-packadd-bang', 'packadd! ' },
    { 'cmd-colorscheme', 'colorscheme ' },
    { 'cmd-scriptnames', 'scriptnames ' },
  }) do
    local label = label_once('k05/' .. spec[1])
    reset()
    local ok, res = pcall(vim.fn.getcompletion, spec[2], 'cmdline')
    if ok then
      report_list(label, res)
    else
      emit(label, '!', esc(scrub(errtext(res))))
    end
  end
end)

-- =====================================================================
-- s06 -- :source.
-- =====================================================================

section('s06-source', function()
  for _, spec in ipairs({
    { 'plain', 'source ' .. work .. '/src/plain.vim' },
    { 'plain-so', 'so ' .. work .. '/src/plain.vim' },
    { 'nested', 'source ' .. work .. '/src/nest.vim' },
    { 'finish', 'source ' .. work .. '/src/fin.vim' },
    { 'continuation', 'source ' .. work .. '/src/cont.vim', { ask = 'g:CONT' } },
    { 'heredoc', 'source ' .. work .. '/src/here.vim', { ask = 'g:HERE' } },
    { 'scriptencoding', 'source ' .. work .. '/src/enc.vim', { ask = 'g:ENC' } },
    { 'error', 'source ' .. work .. '/src/err.vim' },
    { 'lua', 'source ' .. work .. '/src/plain.lua' },
    { 'lua-error', 'source ' .. work .. '/src/err.lua' },
    { 'crlf', 'source ' .. work .. '/src/crlf.vim', { ask = 'g:CRLF' } },
    { 'bom', 'source ' .. work .. '/src/bom.vim', { ask = 'g:BOM' } },
    { 'noeol', 'source ' .. work .. '/src/noeol.vim' },
    { 'script-local', 'source ' .. work .. '/src/vars.vim', { ask = 'Visible()' } },
    { 'missing', 'source ' .. work .. '/src/nosuch.vim' },
    { 'directory', 'source ' .. work .. '/src' },
    { 'no-arg-empty-buffer', 'source' },
    { 'twice', 'source ' .. work .. '/src/plain.vim | source ' .. work .. '/src/plain.vim' },
    { 'relative', 'source src/plain.vim' },
    { 'tilde', 'source ~/nosuch.vim' },
    { 'wildcard', 'source ' .. work .. '/src/pl*.vim' },
    { 'luafile', 'luafile ' .. work .. '/src/plain.lua' },
    { 'runtime-vim-lua', 'runtime! plugin/p1.*' },
  }) do
    case('k06/' .. spec[1], spec[2], spec[3])
  end

  -- :source of a BUFFER, and of a RANGE of one: a different entry point
  -- (`cmd_source_buffer` / `do_source_buffer_init`) from the file path.
  for _, spec in ipairs({
    { 'buf-whole', 'source', 'x.vim' },
    { 'buf-range', '2,3source', 'x.vim' },
    { 'buf-one-line', '1source', 'x.vim' },
    { 'buf-percent', '%source', 'x.vim' },
    { 'buf-reversed', '3,2source', 'x.vim' },
    { 'buf-lua', 'source', 'x.lua' },
    { 'buf-lua-range', '2,2source', 'x.lua' },
  }) do
    local label = label_once('k06/' .. spec[1])
    reset()
    local lines = spec[3]:match('%.lua$')
        and {
          'vim.cmd("call add(g:RTM, \'buf-lua-1\')")',
          'vim.cmd("call add(g:RTM, \'buf-lua-2\')")',
          'vim.cmd("call add(g:RTM, \'buf-lua-3\')")',
        }
      or {
        "call add(g:RTM, 'buf-1')",
        "call add(g:RTM, 'buf-2')",
        "call add(g:RTM, 'buf-3')",
      }
    vim.api.nvim_buf_set_lines(0, 0, -1, false, lines)
    vim.api.nvim_buf_set_name(0, work .. '/' .. spec[3])
    local ok, err = pcall(vim.cmd, spec[2])
    if not ok then
      emit(label, '!', esc(scrub(errtext(err))))
    end
    report_markers(label)
    vim.api.nvim_buf_set_lines(0, 0, -1, false, {})
  end

  -- do_source_str: the same machinery with no file behind it.
  for _, spec in ipairs({
    { 'exec-str', "call add(g:RTM, 'exec-str')" },
    { 'exec-str-finish', "call add(g:RTM, 'a')\nfinish\ncall add(g:RTM, 'b')" },
    { 'exec-str-sfile', "let g:SFX = expand('<sfile>')" },
    { 'exec-str-error', 'call NoSuchFn()' },
  }) do
    local label = label_once('k06/' .. spec[1])
    reset()
    local ok, res = pcall(vim.api.nvim_exec2, spec[2], { output = true })
    emit(label, 'O', esc(scrub(ok and (res.output or '') or ('! ' .. errtext(res)))))
    report_markers(label)
  end
  ask(label_once('k06/exec-str-sfile-value'), 'get(g:, "SFX", "<unset>")')
end)

-- =====================================================================
-- s07 -- the execution stack.
-- =====================================================================

section('s07-estack', function()
  reset()
  vim.cmd('source ' .. work .. '/src/sfile.vim')
  ask(label_once('k07/sfile-at-top-of-script'), 'g:SF')

  reset()
  vim.cmd('source ' .. work .. '/src/func.vim')
  ask(label_once('k07/inside-function'), 'Deep()')
  ask(label_once('k07/nested-function'), 'Outer()')

  reset()
  vim.cmd('source ' .. work .. '/src/trace.vim')
  ask(label_once('k07/stacktrace-three-deep'), 'T1()')
  ask(label_once('k07/stacktrace-two-deep'), 'T2()')
  ask(label_once('k07/stacktrace-one-deep'), 'T3()')

  for _, spec in ipairs({
    { 'expand-sfile', "expand('<sfile>')" },
    { 'expand-script', "expand('<script>')" },
    { 'expand-slnum', "expand('<slnum>')" },
    { 'expand-sflnum', "expand('<sflnum>')" },
    { 'expand-stack', "expand('<stack>')" },
    { 'stacktrace-top', 'getstacktrace()' },
  }) do
    reset()
    ask(label_once('k07/' .. spec[1]), spec[2])
  end

  -- The same five, evaluated from inside a sourced script rather than
  -- from the top level -- which is where they have an answer at all.
  local probe = work .. '/src/probe.vim'
  for _, spec in ipairs({
    { 'in-script-sfile', "expand('<sfile>')" },
    { 'in-script-script', "expand('<script>')" },
    { 'in-script-slnum', "expand('<slnum>')" },
    { 'in-script-sflnum', "expand('<sflnum>')" },
    { 'in-script-stack', "expand('<stack>')" },
    { 'in-script-stacktrace', 'getstacktrace()' },
    { 'in-script-sourced-lnum', "expand('<slnum>') .. ':' .. expand('<sflnum>')" },
  }) do
    local label = label_once('k07/' .. spec[1])
    reset()
    local fd = assert(io.open(probe, 'w'))
    fd:write('" a comment, so <slnum> is not 1\nlet g:P = ' .. spec[2] .. '\n')
    fd:close()
    vim.cmd('source ' .. probe)
    ask(label, 'g:P')
  end
  os.remove(probe)
end)

-- =====================================================================
-- s08 -- the script registry.
-- =====================================================================

--- getscriptinfo() with the volatile parts folded: the whole list is a
--- registry golden, so the *count* and the order matter as much as any
--- one row.
local function scriptinfo(label, arg)
  -- No argument at all is a different call from a null one: passing
  -- `nil` through vim.fn makes it v:null and raises E1206.
  local ok, res
  if arg == nil then
    ok, res = pcall(vim.fn.getscriptinfo)
  else
    ok, res = pcall(vim.fn.getscriptinfo, arg)
  end
  if not ok then
    emit(label, '!', esc(scrub(errtext(res))))
    return
  end
  local rows = {}
  for i, item in ipairs(res) do
    rows[i] = string.format(
      '%d:%s%s',
      item.sid or -1,
      (tostring(item.name or ''):gsub(vim.pesc(work) .. '/?', '')),
      item.autoload and ':autoload' or ''
    )
  end
  emit(label, 'R', esc(scrub(table.concat(rows, ' '))))
  struct(label .. '#R', rows)
end

section('s08-script', function()
  reset()
  vim.cmd('source ' .. work .. '/src/plain.vim')
  vim.cmd('source ' .. work .. '/src/plain.lua')
  vim.cmd('source ' .. work .. '/src/vars.vim')

  scriptinfo(label_once('k08/getscriptinfo-all'))
  emit(
    label_once('k08/scriptnames'),
    'O',
    esc(scrub(vim.api.nvim_exec2('scriptnames', { output = true }).output or ''))
  )

  -- Look the fixture's own SID up by name and then ask for it by
  -- number: `find_script_by_name` and the {'sid'} filter are different
  -- code paths onto the same table.
  local sid
  for _, item in ipairs(vim.fn.getscriptinfo()) do
    if tostring(item.name):match('src/plain%.vim$') then
      sid = item.sid
    end
  end
  emit(label_once('k08/plain-sid'), 'A', 'sid=' .. tostring(sid))
  scriptinfo(label_once('k08/by-sid'), { sid = sid })
  scriptinfo(label_once('k08/by-sid-zero'), { sid = 0 })
  scriptinfo(label_once('k08/by-sid-huge'), { sid = 99999 })
  scriptinfo(label_once('k08/by-name-plain'), { name = 'plain' })
  scriptinfo(label_once('k08/by-name-anchored'), { name = 'src/plain\\.vim$' })
  scriptinfo(label_once('k08/by-name-nomatch'), { name = 'zzzznosuch' })
  scriptinfo(label_once('k08/by-name-empty'), { name = '' })
  scriptinfo(label_once('k08/by-both'), { sid = sid, name = 'zzz' })
  scriptinfo(label_once('k08/bad-key'), { nosuchkey = 1 })

  -- The script-local function list only exists for a {'sid'} query.
  local label = label_once('k08/sid-detail')
  local ok, res = pcall(vim.fn.getscriptinfo, { sid = sid })
  emit(label, 'A', esc(scrub(ok and ins(res) or ('! ' .. errtext(res)))))
  struct(label .. '#A', ok and res or ('! ' .. errtext(res)))

  -- Autoload: which file resolves the name, and what a second call does.
  for _, spec in ipairs({
    { 'autoload-hit', 'al#f()' },
    { 'autoload-twice', 'al#f() . "/" . al#f()' },
    { 'autoload-nested', 'nest#ed#g()' },
    { 'autoload-miss', 'nosuch#f()' },
    { 'autoload-miss-nested', 'no#such#f()' },
    { 'autoload-empty-head', '#f()' },
    { 'autoload-trailing-hash', 'al#()' },
    { 'autoload-reversed-rtp', 'al2#f()' },
  }) do
    local lbl = label_once('k08/' .. spec[1])
    reset(spec[1] == 'autoload-reversed-rtp' and { 'rt/d2', 'rt/d1' } or nil)
    ask(lbl, spec[2])
    report_markers(lbl)
  end

  scriptinfo(label_once('k08/getscriptinfo-after-autoload'))

  for _, spec in ipairs({
    { 'finish-outside', 'finish' },
    { 'scriptencoding-outside', 'scriptencoding latin1' },
    { 'scriptnames-bad-arg', 'scriptnames 1' },
  }) do
    case('k08/' .. spec[1], spec[2])
  end
end)

-- =====================================================================
-- s20 -- the uncaptured block.  This is the whole of .stderr.
-- =====================================================================

-- The commands whose ANSWER is a message.  They run in children, with
-- one `-c` each, because that is the only spelling under which nvim
-- *displays* an error and keeps going: a `pcall` around `vim.cmd`
-- catches the error before `emsg` ever sees it (which is why an earlier
-- draft of this section produced an artifact with no E-number in it),
-- and an unprotected one takes the whole run down.  `--cmd` and `-c` are
-- capped at ten entries each (MAX_ARG_CMDS), hence the batching.
local MSG_CMDS = {
  'runtime plugin/nosuchfile.vim',
  'runtime! plugin/nosuchfile.vim',
  'runtime',
  'runtime!',
  'runtime START',
  'packadd nosuchpack',
  'packadd',
  'packadd!',
  'source ' .. work .. '/src/nosuch.vim',
  'source ' .. work .. '/src',
  'source ' .. work .. '/src/err.vim',
  'source ' .. work .. '/src/err.lua',
  'finish',
  'scriptencoding nosuchencoding',
  'call nosuch#f()',
  'call al#nosuchmember()',
  'echo getscriptinfo({"sid": "notanumber"})',
  'echo getscriptinfo("notadict")',
  'echo getstacktrace(1)',
  'scriptnames 1',
  'packloadall nosucharg',
}

section('s20-messages', function()
  local rtp = table.concat(abs(RTP), ',')
  local pp = table.concat(abs(PACKP), ',')
  local i = 1
  while i <= #MSG_CMDS do
    local args = {
      work .. '/bin/nvim',
      '--headless',
      '-u',
      'NONE',
      '-i',
      'NONE',
      '--cmd',
      'set runtimepath=' .. rtp,
      '--cmd',
      'set packpath=' .. pp,
      -- Every fixture script appends to it; without it each one reports
      -- E121 + E116 and buries the message the case is actually about.
      '--cmd',
      'let g:RTM = []',
    }
    for _ = 1, 9 do
      if MSG_CMDS[i] then
        vim.list_extend(args, { '-c', MSG_CMDS[i] })
        i = i + 1
      end
    end
    vim.list_extend(args, { '-c', 'qa!' })
    local res = vim
      .system(args, { text = true, cwd = work, env = BASE_ENV, clear_env = true, timeout = 60000 })
      :wait()
    io.stderr:write(scrub(res.stderr or ''))
  end

  -- The in-process half: messages that are NOT errors reach stderr from
  -- here perfectly well, and this is where the cumulative script list
  -- lives.  NOT `verbose=0`: batch mode starts 'verbose' at 1 and that
  -- is what routes nvim's messages to stderr at all.  Zero silences the
  -- lot and leaves this artifact empty while every other one looks
  -- healthy.
  reset()
  vim.o.verbose = 1
  local function try(cmd)
    pcall(vim.cmd, cmd)
  end
  try('scriptnames')
  -- 'verbose' at 2 is what makes do_source_ext announce each file it
  -- opens, and at 11 what makes do_in_cached_path announce each search:
  -- the only view of either message there is.
  vim.o.verbose = 2
  try('source ' .. work .. '/src/plain.vim')
  try('runtime! plugin/p1.vim')
  try('packadd o1')
  vim.o.verbose = 12
  try('runtime plugin/p2.vim')
  try('runtime START plugin/s1.vim')
  vim.o.verbose = 1
  try('scriptnames')
end)

-- =====================================================================
-- s91 -- CRASHPROBE.
-- =====================================================================

section('s91-crashprobe', function()
  local progpath = work .. '/bin/nvim'
  local i, guard = 1, 0
  local aborted = 0
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
        '--crash',
        tostring(i),
        -- A safety net, not the assertion: every input here is meant to
        -- finish or die.  A child that hits the timeout is killed and
        -- reads as ABORTED, so anything that *wedges* has to be trimmed
        -- out of the corpus instead.
      }, { text = true, cwd = work, env = BASE_ENV, clear_env = true, timeout = 120000 })
      :wait()
    local last = i - 1
    for line in (res.stdout or ''):gmatch('[^\n]+') do
      local idx = tonumber(line:match('^(%d+) '))
      if idx then
        last = idx
        emit((line:gsub('^%d+ ', '')))
      end
    end
    -- vim.system reports an abort as a SIGNAL and leaves `code` at 0, so
    -- a code-only check calls a SIGABRT a success.
    local died = (res.signal or 0) ~= 0 or (res.code or 0) ~= 0
    if not died and last >= #CRASH then
      break
    end
    if died then
      local victim = CRASH[last + 1]
      if victim then
        aborted = aborted + 1
        emit(
          'k91/' .. victim[1],
          'X',
          string.format('ABORTED code=%s signal=%s', tostring(res.code), tostring(res.signal))
        )
      end
      last = last + 1
    end
    i = last + 1
  end
  emit('k91', 'groups', string.format('cases=%d aborted=%d', #CRASH, aborted))
  struct('k91#groups', { cases = #CRASH, aborted = aborted })
end)

if structfd then
  structfd:close()
end
emit('## done')
