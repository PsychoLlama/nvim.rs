-- Driver for the typeahead/mapping differential sweep; see
-- keysweep.sh.
--
-- Covers the three input modules of batch B13:
--
--   mapping    mapping.rs   -- do_map/buf_do_map, the listing text,
--                              mapblock_fill_dict's twenty keys through
--                              maparg()/mapcheck()/maplist()/mapset()/
--                              nvim_{,buf_}{get,set,del}_keymap, the
--                              <expr>/<silent>/<nowait>/<buffer>/<script>/
--                              <unique>/<Plug>/<Cmd> flags, langmap and
--                              the abbreviation half.
--   getchar    getchar.rs   -- vgetorpeek/vgetc/handle_mapping: the
--                              'timeout'/'ttimeout'/'timeoutlen'/
--                              'ttimeoutlen' matrix over an ambiguous
--                              mapping, feedkeys()'s mode letters,
--                              getchar()/getcharstr()/getcharmod(),
--                              register record/replay and :normal.
--   keycodes   keycodes.rs  -- the key-name tables, both directions:
--                              replace_termcodes/keytrans/"\<Key>" over
--                              a corpus of every notation family, plus
--                              what each one looks like once it is a
--                              mapping's lhsraw.
--
-- TWO DRIVERS, and the split is forced, not stylistic.
--
--   * In-process (`-l`).  Everything that is a question about *state*:
--     listings, dict answers, notation, and any key sequence that
--     resolves without waiting.  `feedkeys(..., 'x')` is the vehicle.
--   * A child `--embed --headless` nvim over RPC (`section 11-12`).
--     `feedkeys(..., 'x')` runs with `ex_normal_busy` set, and in that
--     state `vgetorpeek` never waits for more input: an ambiguous
--     mapping is resolved to the short match *immediately*, whatever
--     'timeout' says.  Measured: all four arms of the timeout matrix
--     answer identically in-process.  Only a process running its own
--     main loop can be made to wait, so the timeout matrix is driven by
--     typing at a child with `nvim_input` and sampling it.
--
-- Sampling a child that is *blocked* mid-mapping is itself constrained:
-- an RPC request that is not marked `fast` is queued to the main loop
-- and is never answered while `vgetorpeek` waits, which deadlocks the
-- parent (measured -- it cost a 124).  `nvim_get_mode()` is `fast` and
-- is therefore the only probe used before a case has come to rest --
-- and `blocking = true` is exactly the assertion the timeout cases
-- want, so this is a feature: "it is still waiting" is a recorded
-- answer, and a hang is a diff rather than a wedged harness.
--
-- Everything printed has to be reproducible across two builds run
-- minutes apart, so the report carries no duration, pid, wall-clock
-- time or path outside the work directory.  Three artifacts: the
-- readable report on stdout, a canonical (sorted-key) JSON dump on
-- $KEY_STRUCT of every dict-shaped answer, and stderr -- which in a
-- headless process is where nvim's own messages go, and is the only
-- view of some of them.

local work = assert(os.getenv('KEY_WORK'), 'KEY_WORK unset')
local structpath = assert(os.getenv('KEY_STRUCT'), 'KEY_STRUCT unset')
local structfd = assert(io.open(structpath, 'w'))
local only = os.getenv('KEYSWEEP_ONLY')
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

--- Strip the bits of an answer that name where the run happened.
local function scrub(text)
  text = tostring(text)
  text = text:gsub(vim.pesc(script), '<SCRIPT>'):gsub('%.%.%.[^%s\'"]-keysweep%.lua', '<SCRIPT>')  -- LuaJIT elides a chunk name past ~60 chars
  text = text:gsub(vim.pesc(work), '<WORK>')
  text = text:gsub(vim.pesc(work:sub(2)), '<WORK>')
  if runtime ~= '' then
    text = text:gsub(vim.pesc(runtime), '<RUNTIME>')
  end
  -- `:undo`'s "N seconds ago" ticks over between two runs of the same
  -- binary.  The same mask is applied to the stderr artifact by
  -- keysweep.sh, because the message reaches both.
  text = text:gsub('%d+ seconds? ago', '<AGO>')
  text = text:gsub('%d+ minutes? ago', '<AGO>')
  return text
end

--- Escape to one printable line, so a byte difference shows in the diff.
local function esc(bytes)
  return (tostring(bytes):gsub('[^\32-\126]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

-- ---------------------------------------------------------------------
-- Canonical dump.  vim.json.encode walks a Lua table in hash order,
-- which is stable within a build but is not something to bet a byte
-- oracle on; keys are sorted here instead.  Empty tables are tagged so
-- an empty list and an empty dict stay distinguishable -- maparg()
-- answers both, and they are different answers.
-- ---------------------------------------------------------------------
local function canon(value)
  local kind = type(value)
  if value == vim.NIL then
    return 'null'
  elseif kind == 'number' then
    return value == math.floor(value) and string.format('%d', value)
      or string.format('%.14g', value)
  elseif kind == 'boolean' then
    return tostring(value)
  elseif kind == 'string' then
    return '"' .. esc(scrub(value)):gsub('"', '\\"') .. '"'
  elseif kind ~= 'table' then
    -- A mapping's `callback` is a function; its identity is not stable,
    -- but its presence is exactly what has to compare equal.
    return '"<' .. kind .. '>"'
  end
  if next(value) == nil then
    return vim.islist(value) and '[]' or '{}'
  end
  if vim.islist(value) then
    local parts = {}
    for _, item in ipairs(value) do
      parts[#parts + 1] = canon(item)
    end
    return '[' .. table.concat(parts, ',') .. ']'
  end
  local keys = {}
  for key in pairs(value) do
    keys[#keys + 1] = tostring(key)
  end
  table.sort(keys)
  local parts = {}
  for _, key in ipairs(keys) do
    parts[#parts + 1] = '"' .. esc(key) .. '":' .. canon(value[key])
  end
  return '{' .. table.concat(parts, ',') .. '}'
end

local function struct(label, value)
  structfd:write(label, ' ', canon(value), '\n')
end

-- ---------------------------------------------------------------------
-- Running things.  An error is observable behaviour: a case that fails
-- has to fail the same way in both binaries, so it is reported, not
-- swallowed.
-- ---------------------------------------------------------------------
local function exec(cmd)
  local ok, res = pcall(vim.api.nvim_exec2, cmd, { output = true })
  if ok then
    return scrub(res.output or '')
  end
  return 'ERROR ' .. scrub(tostring(res):gsub('.-:%s*Vim', 'Vim'))
end

local function run(label, cmd)
  local out = exec(cmd)
  if out ~= '' then
    emit(label, '|', (esc(out):gsub('\\x0a', '\n' .. label .. ' | ')))
  else
    emit(label, '| (silent)')
  end
end

--- Evaluate a Vimscript expression and report both forms.
local function evalp(label, expr)
  local ok, res = pcall(vim.fn.eval, expr)
  if not ok then
    local text = 'ERROR ' .. scrub(tostring(res):gsub('.-:%s*Vim', 'Vim'))
    emit(label, '=', esc(text))
    struct(label, text)
    return nil
  end
  emit(label, '=', esc((scrub(vim.inspect(res)):gsub('%s+', ' '))))
  struct(label, res)
  return res
end

--- Call a Vimscript function, answering the error text rather than
--- dying: a rejected argument list is observable behaviour and has to
--- compare equal too.
local function vcall(name, ...)
  local ok, res = pcall(vim.fn[name], ...)
  if ok then
    return res
  end
  return 'ERROR ' .. scrub(tostring(res):gsub('.-:%s*E', 'E'))
end

--- Report and record one answer under a label.
local function answer(label, value)
  emit(label, '=', esc((scrub(vim.inspect(value)):gsub('%s+', ' '))))
  struct(label, value)
end

local function tc(keys)
  return vim.api.nvim_replace_termcodes(keys, true, true, true)
end

--- Feed keys as if typed, then let the caller look at the damage.
--- 'x' so the keys are consumed before this returns; 't' so mappings
--- and abbreviations behave as they would interactively.
local function feed(keys, mode)
  return pcall(vim.fn.feedkeys, tc(keys), mode or 'xt')
end

local DEFAULT_LINES = { 'alpha beta gamma', 'second line here', 'third line' }

--- Drop every mapping and abbreviation, in every mode, global and
--- buffer-local.  Called before each *case*, not just each section: a
--- map that survives into the next case makes that case's answer a
--- function of the order the cases happen to be written in, which is
--- exactly the property a differential oracle must not have.
local function clear_maps()
  for _, prefix in ipairs({ '', 'n', 'v', 'x', 's', 'o', 'i', 'l', 'c', 't' }) do
    exec('silent! ' .. prefix .. 'mapclear')
    exec('silent! ' .. prefix .. 'mapclear <buffer>')
  end
  exec('silent! mapclear!')
  exec('silent! mapclear! <buffer>')
  exec('silent! abclear')
  exec('silent! abclear <buffer>')
  exec('silent! iabclear')
  exec('silent! cabclear')
end

--- Reset to a known editor state between sections.  Every option this
--- batch reads is set explicitly: a sweep that inherits one is a sweep
--- whose baseline moves when a default does.
local function reset()
  clear_maps()
  if vim.fn.reg_recording() ~= '' then
    feed('q')
  end
  exec('silent! %bwipeout!')
  exec('silent! enew!')
  local opts = {
    timeout = true,
    ttimeout = true,
    timeoutlen = 1000,
    ttimeoutlen = 50,
    maxmapdepth = 1000,
    langmap = '',
    langremap = false,
    cpoptions = 'aABceFs_',
    wildchar = 9,
    wildcharm = 0,
    report = 9999,
    more = false,
    shortmess = 'filnxtToOF',
    swapfile = false,
    showcmd = false,
    showmode = false,
    ruler = false,
    lazyredraw = false,
    autoindent = false,
    expandtab = false,
    tabstop = 8,
    softtabstop = 0,
    shiftwidth = 8,
    textwidth = 0,
    backspace = 'indent,eol,start',
    virtualedit = '',
    selection = 'inclusive',
    startofline = true,
    whichwrap = 'b,s',
    iskeyword = '@,48-57,_,192-255',
    isprint = '@,161-255',
    encoding = 'utf-8',
    fileformats = 'unix',
    iminsert = 0,
    imsearch = -1,
    scrolloff = 0,
    sidescrolloff = 0,
    undolevels = 1000,
  }
  for name, value in pairs(opts) do
    pcall(function()
      vim.o[name] = value
    end)
  end
  for _, reg in ipairs({ 'q', 'r', 'a', '"' }) do
    pcall(vim.fn.setreg, reg, '')
  end
  vim.v.errmsg = ''
  pcall(vim.cmd, 'silent! cd ' .. vim.fn.fnameescape(work))
end

local function heading(text)
  emit('')
  emit('===== ' .. text .. ' =====')
end

--- A section runs after a `reset()` and never lets an error kill the
--- run: a section that throws on one side and not on the other is the
--- loudest possible diff, and the remaining sections still get taken.
local sections_run = 0
local function section(name, fn)
  if only and not name:match(only) then
    return
  end
  sections_run = sections_run + 1
  heading(name)
  reset()
  local ok, err = pcall(fn)
  if not ok then
    emit(name, '!!', 'SECTION ERROR', esc(scrub(tostring(err))))
  end
  reset()
end

-- ---------------------------------------------------------------------
-- One keystroke scenario, recorded whole.
--
-- `redir` is used rather than the stderr artifact alone because it
-- labels the messages: "recording @q", "-- INSERT --" and every error
-- reach stderr unlabelled and interleaved, and this way the case that
-- produced them is named.  They still reach stderr, so nothing is lost.
-- ---------------------------------------------------------------------
local function keycase(label, prep, keys, opts)
  opts = opts or {}
  clear_maps()
  exec('silent! enew!')
  pcall(vim.api.nvim_buf_set_lines, 0, 0, -1, false, opts.lines or DEFAULT_LINES)
  pcall(vim.api.nvim_win_set_cursor, 0, opts.cursor or { 1, 0 })
  for _, cmd in ipairs(prep or {}) do
    exec('silent! ' .. cmd)
  end
  vim.v.errmsg = ''
  vim.g.keymsg = ''
  pcall(vim.cmd, 'redir => g:keymsg')
  local ok, err = pcall(function()
    if opts.via == 'normal' then
      vim.api.nvim_exec2('normal ' .. keys, { output = false })
    elseif opts.via == 'normal!' then
      vim.api.nvim_exec2('normal! ' .. keys, { output = false })
    elseif opts.via == 'exec' then
      vim.api.nvim_exec2(keys, { output = false })
    else
      vim.fn.feedkeys(tc(keys), opts.mode or 'xt')
    end
  end)
  pcall(vim.cmd, 'redir END')
  local recording = vim.fn.reg_recording()
  local state = {
    keys = keys,
    via = opts.via or ('feedkeys ' .. (opts.mode or 'xt')),
    lines = vim.api.nvim_buf_get_lines(0, 0, -1, false),
    mode = vim.api.nvim_get_mode().mode,
    errmsg = vim.v.errmsg,
    err = ok and '' or scrub(tostring(err):gsub('.-:%s*Vim', 'Vim')),
    msg = scrub(vim.g.keymsg or ''),
    recording = recording,
  }
  local okc, cur = pcall(vim.api.nvim_win_get_cursor, 0)
  state.cursor = okc and { cur[1], cur[2] + 1 } or { 0, 0 }
  for _, reg in ipairs(opts.regs or {}) do
    state['reg_' .. reg] = vcall('getreg', reg)
  end
  for _, name in ipairs(opts.vars or {}) do
    state['g_' .. name] = vim.g[name] == nil and vim.NIL or vim.g[name]
  end
  struct(label, state)
  emit(
    label,
    '>',
    esc(keys),
    '|',
    esc(vim.inspect(state.lines):gsub('%s+', ' ')),
    '| cur=' .. state.cursor[1] .. ',' .. state.cursor[2],
    '| mode=' .. esc(state.mode),
    '| err=' .. esc(state.errmsg ~= '' and state.errmsg or state.err),
    '| msg=' .. esc(state.msg)
  )
  -- Leave normal mode however the case ended, so the next case starts
  -- where it thinks it does.
  if recording ~= '' then
    feed('q')
  end
  pcall(vim.cmd, 'silent! stopinsert')
  feed('<Esc>')
end

reset()

-- =====================================================================
-- 1. The mappings nvim ships with.
--
-- Taken before anything is cleared.  This is the widest single exercise
-- of mapblock_fill_dict in the sweep: the defaults are the only maps in
-- the tree that carry `desc`, a Lua `callback`, `replace_keycodes`, a
-- `<Cmd>` rhs and an `lhsrawalt` (the CSI-vs-control-byte alternative
-- spelling of a <C-x> lhs) all at once.  `lnum`/`sid` name a runtime
-- file, so this section moves if runtime/lua/vim/_defaults.lua does --
-- which is the correct answer, not a false positive.
-- =====================================================================
if not only or ('defaults'):match(only) then
  heading('defaults')
  for _, mode in ipairs({ 'n', 'i', 'c', 'v', 'x', 's', 'o', 't', 'l', '' }) do
    local maps = vcall('maplist')
    local got = vim.api.nvim_get_keymap(mode)
    struct('defaults get_keymap ' .. (mode == '' and '<all>' or mode), got)
    emit('defaults get_keymap', mode == '' and '<all>' or mode, '=', #got, 'entries')
    if mode == 'n' then
      struct('defaults maplist', maps)
      emit('defaults maplist =', type(maps) == 'table' and #maps or 0, 'entries')
    end
  end
end

-- =====================================================================
-- 2. mapping.rs -- the listing text.
--
-- `:map` output is `showmap()` over the whole table plus `str2special`
-- on both halves, so it is simultaneously the mapping oracle and a
-- second view of the keycode tables.  The corpus below is chosen so
-- that every flag column of the listing (`*`, `&`, `@`, the mode
-- letter) is exercised and so that the sort order has ties to break.
-- =====================================================================
local MAP_CORPUS = {
  'nmap ,g gg',
  'nnoremap ,n gg',
  'nmap <silent> ,s :echo "s"<CR>',
  'nmap <nowait> ,w x',
  'nmap ,wide xxxxxx',
  [[nnoremap <expr> ,e '"iX\<Esc>"']],
  'nnoremap <buffer> ,b dd',
  'nmap <script> ,c yy',
  'nmap <Plug>Foo yy',
  'nmap ,p <Plug>Foo',
  'nnoremap ,C <Cmd>let g:cmd = 1<CR>',
  'map ,m x',
  'map! ,x y',
  'vmap ,v y',
  'xmap ,X y',
  'smap ,S y',
  'omap ,o iw',
  'cmap ,q <C-a>',
  'tmap ,t <C-w>',
  'lmap ,l z',
  'noremap ,N x',
  'imap jk <Esc>',
  'nmap <C-x><C-y> ZZ',
  'nmap <M-a> x',
  'nmap <F5> x',
  'nmap <Space><Tab> x',
  'nmap <lt>x> x',
  'nmap <Bar>z x',
  'nmap \\d :echo "d"<CR>',
  'nmap ,u <Nop>',
  'iabbrev teh the',
  'cabbrev cwd /tmp',
  'abbrev ab abbreviation',
  'noreabbrev nab noremapped',
  'inoreabbrev iab inoremapped',
}

section('map-listing', function()
  for _, cmd in ipairs(MAP_CORPUS) do
    run('define', cmd)
  end
  for _, cmd in ipairs({
    'map',
    'map!',
    'nmap',
    'imap',
    'vmap',
    'xmap',
    'smap',
    'omap',
    'cmap',
    'tmap',
    'lmap',
    'noremap',
    'map ,',
    'map ,g',
    'nmap ,g',
    'map <buffer>',
    'nmap <buffer>',
    'map <Plug>',
    'map <C-x>',
    'abbrev',
    'iabbrev',
    'cabbrev',
    'map ,zz',
    'verbose map ,g',
    'verbose nmap ,b',
  }) do
    run('list ' .. cmd, cmd)
  end
end)

section('map-errors', function()
  for _, cmd in ipairs(MAP_CORPUS) do
    exec('silent! ' .. cmd)
  end
  for _, cmd in ipairs({
    'unmap ,g',
    'unmap ,g',
    'unmap ,zz',
    'nunmap ,n',
    'iunmap jk',
    'iunmap jk',
    'unabbrev teh',
    'iunabbrev teh',
    'cunabbrev cwd',
    'unmap <buffer> ,b',
    'unmap <buffer> ,b',
    'nmap <unique> ,U x',
    'nmap <unique> ,U y',
    'nmap <unique> ,m y',
    'map',
    'map <expr>',
    'map <silent>',
    'map <nowait>',
    'map <buffer>',
    'noremap <script>',
    'nmap ,bad',
    'nmap <NotAKey> x',
    'nmap <C-> x',
    'nmap <> x',
    'nmap <expr> <buffer> ,eb 1',
    'unmap',
    'mapclear <buffer> extra',
    'abbrev a b c',
    'iabbrev <expr> ex 1+1',
  }) do
    run('err ' .. cmd, cmd)
  end
end)

-- =====================================================================
-- 3. mapping.rs -- the dict surface.
--
-- The twenty keys mapblock_fill_dict can set are: abbr buf buffer
-- callback desc expr lhs lhsraw lhsrawalt lnum mode mode_bits noremap
-- nowait replace_keycodes rhs script scriptversion sid silent.  No one
-- map carries all of them; the corpus below is picked so the union
-- does.  This is the artifact half the readable report cannot see: a
-- dropped `lhsrawalt` or a wrong `mode_bits` prints the same listing.
-- =====================================================================
local DICT_PROBES = {
  { ',g', 'n', false },
  { ',n', 'n', false },
  { ',s', 'n', false },
  { ',w', 'n', false },
  { ',e', 'n', false },
  { ',b', 'n', false },
  { ',c', 'n', false },
  { ',p', 'n', false },
  { ',C', 'n', false },
  { ',m', 'n', false },
  { ',m', 'v', false },
  { ',m', 'o', false },
  { ',x', 'i', false },
  { ',x', 'c', false },
  { ',v', 'v', false },
  { ',X', 'x', false },
  { ',S', 's', false },
  { ',o', 'o', false },
  { ',q', 'c', false },
  { ',t', 't', false },
  { ',l', 'l', false },
  { 'jk', 'i', false },
  { '<C-x><C-y>', 'n', false },
  { '<M-a>', 'n', false },
  { '<F5>', 'n', false },
  { '<Space><Tab>', 'n', false },
  { ',u', 'n', false },
  { ',g', 'i', false },
  { ',nothere', 'n', false },
  { 'teh', 'i', true },
  { 'cwd', 'c', true },
  { 'ab', 'i', true },
  { 'nab', 'i', true },
  { 'iab', 'i', true },
}

section('map-dict', function()
  for _, cmd in ipairs(MAP_CORPUS) do
    exec('silent! ' .. cmd)
  end
  for _, probe in ipairs(DICT_PROBES) do
    local lhs, mode, abbr = probe[1], probe[2], probe[3]
    local key = ('maparg %q %s%s'):format(lhs, mode, abbr and ' abbr' or '')
    answer(key .. ' str', vcall('maparg', lhs, mode, abbr, false))
    answer(key .. ' dict', vcall('maparg', lhs, mode, abbr, true))
  end
  for _, probe in ipairs({ ',', ',g', ',gg', ',e', ',z', 'j', 'jk', '<F5>', '' }) do
    for _, mode in ipairs({ 'n', 'i', 'c', 'v' }) do
      answer(('mapcheck %q %s'):format(probe, mode), vcall('mapcheck', probe, mode))
      answer(('mapcheck %q %s abbr'):format(probe, mode), vcall('mapcheck', probe, mode, true))
    end
  end
  for _, rhs in ipairs({ 'gg', 'x', '<Plug>Foo', 'yy', 'dd', 'nothing' }) do
    for _, mode in ipairs({ 'n', 'i', '', 'v' }) do
      answer(('hasmapto %q %q'):format(rhs, mode), vcall('hasmapto', rhs, mode))
    end
    answer(('hasmapto %q abbr'):format(rhs), vcall('hasmapto', rhs, 'i', true))
  end
  answer('maplist', vcall('maplist'))
  answer('maplist abbr', vcall('maplist', true))
  for _, mode in ipairs({ 'n', 'i', 'c', 'v', 'x', 's', 'o', 't', 'l', '' }) do
    local m = mode == '' and '<all>' or mode
    answer('get_keymap ' .. m, vim.api.nvim_get_keymap(mode))
    answer('buf_get_keymap ' .. m, vim.api.nvim_buf_get_keymap(0, mode))
  end
  -- Bad arguments are answers too.
  answer('maparg bad-mode', vcall('maparg', ',g', 'zz'))
  answer('maparg no-args', vcall('maparg'))
  answer('mapcheck bad-mode', vcall('mapcheck', ',g', 'zz'))
  answer('get_keymap bad-mode', (function()
    local ok, res = pcall(vim.api.nvim_get_keymap, 'zz')
    return ok and res or ('ERROR ' .. scrub(tostring(res)))
  end)())
end)

section('map-set', function()
  for _, cmd in ipairs(MAP_CORPUS) do
    exec('silent! ' .. cmd)
  end
  -- mapset() has to reconstruct a mapping from the dict maparg() gave,
  -- flags and all -- the round trip is the assertion.
  for _, probe in ipairs({
    { ',g', 'n', false },
    { ',s', 'n', false },
    { ',e', 'n', false },
    { ',b', 'n', false },
    { ',C', 'n', false },
    { ',w', 'n', false },
    { 'teh', 'i', true },
  }) do
    local lhs, mode, abbr = probe[1], probe[2], probe[3]
    local key = ('mapset %q %s'):format(lhs, mode)
    local saved = vcall('maparg', lhs, mode, abbr, true)
    answer(key .. ' saved', saved)
    exec(('silent! %s%sunmap%s %s'):format(
      mode == 'i' and 'i' or mode,
      abbr and 'un' or '',
      abbr and 'abbrev' or '',
      lhs
    ))
    exec(('silent! %sunmap %s'):format(mode, lhs))
    answer(key .. ' after-unmap', vcall('maparg', lhs, mode, abbr, true))
    answer(key .. ' set', vcall('mapset', saved))
    answer(key .. ' restored', vcall('maparg', lhs, mode, abbr, true))
  end
  answer('mapset three-arg', vcall('mapset', 'n', false, { lhs = ',3', rhs = 'gg' }))
  answer('mapset three-arg read', vcall('maparg', ',3', 'n', false, true))
  answer('mapset bad', vcall('mapset', { lhs = ',4' }))
  answer('mapset empty', vcall('mapset', {}))
end)

section('map-api', function()
  local sets = {
    { 'n', ',A', 'gg', {} },
    { 'n', ',B', 'gg', { noremap = true } },
    { 'n', ',C', 'gg', { silent = true } },
    { 'n', ',D', 'gg', { nowait = true } },
    { 'n', ',E', '"iX"', { expr = true } },
    { 'n', ',F', 'gg', { desc = 'a described map' } },
    { 'n', ',G', 'gg', { replace_keycodes = true, expr = true } },
    { 'n', ',H', 'gg', { script = true, noremap = true } },
    { 'i', ',I', 'x', { noremap = true, silent = true } },
    { '', ',J', 'x', {} },
    { '!', ',K', 'x', {} },
    { 'x', ',L', 'y', {} },
    { 'n', '<C-p><C-q>', 'gg', {} },
  }
  for _, s in ipairs(sets) do
    local mode, lhs, rhs, opts = s[1], s[2], s[3], s[4]
    local ok, err = pcall(vim.api.nvim_set_keymap, mode, lhs, rhs, opts)
    emit('set_keymap', esc(mode), esc(lhs), '->', ok and 'ok' or esc(scrub(tostring(err))))
    answer(('set_keymap %q %q'):format(mode, lhs), vcall('maparg', lhs, mode == '' and 'n' or mode, false, true))
  end
  local okcb = pcall(vim.api.nvim_set_keymap, 'n', ',M', '', {
    callback = function() end,
    desc = 'lua callback',
  })
  emit('set_keymap callback ->', okcb and 'ok' or 'error')
  answer('set_keymap callback dict', vcall('maparg', ',M', 'n', false, true))
  pcall(vim.api.nvim_buf_set_keymap, 0, 'n', ',N', 'gg', { nowait = true })
  answer('buf_set_keymap dict', vcall('maparg', ',N', 'n', false, true))
  answer('buf_get_keymap', vim.api.nvim_buf_get_keymap(0, 'n'))
  for _, bad in ipairs({
    { 'zz', ',O', 'gg', {} },
    { 'n', '', 'gg', {} },
    { 'n', ',P', 'gg', { bogus = true } },
    { 'n', ',Q', 'gg', { expr = 'yes' } },
  }) do
    local ok, err = pcall(vim.api.nvim_set_keymap, bad[1], bad[2], bad[3], bad[4])
    emit('set_keymap bad', esc(bad[1]), esc(bad[2]), '->', ok and 'ok' or esc(scrub(tostring(err))))
  end
  for _, del in ipairs({ { 'n', ',A' }, { 'n', ',A' }, { 'n', ',nothere' }, { '', ',J' } }) do
    local ok, err = pcall(vim.api.nvim_del_keymap, del[1], del[2])
    emit('del_keymap', esc(del[1]), esc(del[2]), '->', ok and 'ok' or esc(scrub(tostring(err))))
  end
  local ok, err = pcall(vim.api.nvim_buf_del_keymap, 0, 'n', ',N')
  emit('buf_del_keymap ->', ok and 'ok' or esc(scrub(tostring(err))))
  answer('after deletes', vim.api.nvim_get_keymap('n'))
end)

-- =====================================================================
-- 4. mapping.rs / getchar.rs -- what the keys actually do.
--
-- Resolution, not timing: every case here has enough input queued that
-- `vgetorpeek` never has to wait, so the answers are the same whether
-- or not this process is running a main loop.  The timing half is
-- section 11.
-- =====================================================================
section('map-behaviour', function()
  keycase('plain', { 'nmap X iA<Esc>' }, 'X')
  keycase('remap-chain', { 'nmap Y X', 'nmap X iA<Esc>' }, 'Y')
  keycase('noremap-blocks', { 'nnoremap Z X', 'nmap X iA<Esc>' }, 'Z')
  -- "If the rhs starts with the lhs the first character is not
  -- remapped" -- the rule that makes `:map x xx` legal.  `xj` rather
  -- than `xx`: the latter is an infinite remap in any Vim, and a sweep
  -- case that never returns is a five-minute timeout, not a finding.
  keycase('rhs-starts-with-lhs', { 'nmap x xj' }, 'x')
  keycase('rhs-starts-with-lhs-2', { 'nmap ,a ,aj' }, ',a')
  keycase('noremap-inner', { 'nmap A B', 'nnoremap B C', 'nmap C iQ<Esc>' }, 'A')
  keycase('recursive-E223', { 'set maxmapdepth=20', 'nmap ,r ,s', 'nmap ,s ,r' }, ',r', { via = 'normal' })
  keycase('recursive-depth-3', { 'set maxmapdepth=3', 'nmap ,r ,s', 'nmap ,s ,r' }, ',r', { via = 'normal' })
  keycase('expr-simple', { [[nnoremap <expr> ,e '"iX\<Esc>"']] }, ',e')
  keycase('expr-count', { 'nnoremap <expr> ,e v:count . "iX\\<Esc>"' }, '3,e')
  keycase('expr-error', { 'nnoremap <expr> ,e Undefined_fn()' }, ',e', { via = 'normal' })
  keycase('expr-empty', { 'nnoremap <expr> ,e ""' }, ',e', { via = 'normal' })
  keycase('expr-replace-keycodes', {
    'lua vim.keymap.set("n", ",k", function() return "iZ<Esc>" end, { expr = true, replace_keycodes = true })',
  }, ',k')
  keycase('expr-no-replace-keycodes', {
    'lua vim.keymap.set("n", ",j", function() return "iZ<Esc>" end, { expr = true, replace_keycodes = false })',
  }, ',j')
  keycase('cmd-mapping', { 'nnoremap ,C <Cmd>let g:cmd = 42<CR>' }, ',C', { vars = { 'cmd' } })
  keycase('cmd-mapping-insert', { 'inoremap ,C <Cmd>let g:cmd = 43<CR>' }, 'i,C', { vars = { 'cmd' } })
  keycase('plug-mapping', { 'nnoremap <Plug>Foo iP<Esc>', 'nmap ,p <Plug>Foo' }, ',p')
  keycase('plug-not-typeable', { 'nnoremap <Plug>Foo iP<Esc>' }, '<Plug>Foo')
  keycase('silent-map', { 'nmap <silent> ,s :echo "hello"<CR>' }, ',s')
  keycase('loud-map', { 'nmap ,s :echo "hello"<CR>' }, ',s')
  keycase('nowait-short', { 'nmap <nowait> ,w iW<Esc>', 'nmap ,wide iL<Esc>' }, ',w')
  keycase('nowait-long-still-typed', { 'nmap <nowait> ,w iW<Esc>', 'nmap ,wide iL<Esc>' }, ',wide')
  keycase('ambiguous-full', { 'nmap ,a iA<Esc>', 'nmap ,ab iB<Esc>' }, ',ab')
  keycase('ambiguous-then-other', { 'nmap ,a iA<Esc>', 'nmap ,ab iB<Esc>' }, ',az')
  keycase('buffer-beats-global', { 'nmap ,b iG<Esc>', 'nmap <buffer> ,b iL<Esc>' }, ',b')
  keycase('buffer-only', { 'nmap <buffer> ,b iL<Esc>' }, ',b')
  keycase('script-map', { 'nmap <script> ,c iS<Esc>' }, ',c')
  keycase('insert-map', { 'imap jk <Esc>' }, 'iabcjkX')
  keycase('cmdline-map', { 'cmap ,q let g:cmd = 7' }, ':,q<CR>', { vars = { 'cmd' } })
  keycase('op-pending-map', { 'omap ,o iw' }, 'd,o')
  keycase('visual-map', { 'vmap ,v d' }, 'v,v')
  keycase('select-map', { 'smap ,S x' }, 'gh,S')
  keycase('mode-nvo-map', { 'map ,m iM<Esc>' }, ',m')
  keycase('count-through-map', { 'nnoremap ,c :let g:cmd = v:count<CR>' }, '5,c', { vars = { 'cmd' } })
  keycase('count-prefix-map', { 'nmap ,d 2x' }, '3,d')
  keycase('unmapped-prefix', { 'nmap ,a iA<Esc>' }, ',,a')
  keycase('nop-map', { 'nmap ,u <Nop>' }, ',uiZ<Esc>')
  keycase('map-to-nothing', { 'nmap ,u <Nop>' }, ',u')
end)

section('map-langmap', function()
  keycase('langmap-off', {}, 'jjiX<Esc>')
  keycase('langmap-pairs', { 'set langmap=jk,kj' }, 'jiX<Esc>')
  keycase('langmap-semicolon', { 'set langmap=ab;xy' }, 'aiX<Esc>')
  keycase('langmap-escaped', { 'set langmap=\\;x' }, ';iX<Esc>')
  keycase('langmap-with-map', { 'set langmap=jk,kj', 'nmap j iJ<Esc>' }, 'j')
  keycase('langremap-on', { 'set langmap=jk,kj', 'set langremap', 'nmap k iK<Esc>' }, 'j')
  keycase('langremap-off', { 'set langmap=jk,kj', 'set nolangremap', 'nmap k iK<Esc>' }, 'j')
  keycase('langmap-insert-unaffected', { 'set langmap=jk,kj' }, 'ijk<Esc>')
  keycase('lmap-iminsert', { 'lmap q w', 'set iminsert=1' }, 'iq<Esc>')
  keycase('lmap-iminsert-off', { 'lmap q w', 'set iminsert=0' }, 'iq<Esc>')
  keycase('lmap-search', { 'lmap q w', 'set imsearch=1' }, '/q<CR>', { lines = { 'aaa', 'w here' } })
  run('langmap listing', 'set langmap?')
end)

section('map-abbrev', function()
  keycase('iabbrev-space', { 'iabbrev teh the' }, 'iteh <Esc>', { lines = { '' } })
  keycase('iabbrev-cr', { 'iabbrev teh the' }, 'iteh<CR><Esc>', { lines = { '' } })
  keycase('iabbrev-ctrl-]', { 'iabbrev teh the' }, 'iteh<C-]><Esc>', { lines = { '' } })
  keycase('iabbrev-no-trigger', { 'iabbrev teh the' }, 'iteh<Esc>', { lines = { '' } })
  keycase('iabbrev-mid-word', { 'iabbrev teh the' }, 'ixteh <Esc>', { lines = { '' } })
  keycase('iabbrev-nonword-lhs', { 'iabbrev #i #include' }, 'i#i <Esc>', { lines = { '' } })
  keycase('iabbrev-full-id', { 'iabbrev a-b ab' }, 'ia-b <Esc>', { lines = { '' } })
  keycase('noreabbrev', { 'inoreabbrev teh the', 'iabbrev the THE' }, 'iteh <Esc>', { lines = { '' } })
  keycase('abbrev-remapped', { 'iabbrev teh the', 'iabbrev the THE' }, 'iteh <Esc>', { lines = { '' } })
  keycase('cabbrev', { 'cabbrev cwd /tmp/here' }, ':let g:cmd = "cwd "<CR>', { vars = { 'cmd' } })
  keycase('cabbrev-no-expand', { 'cabbrev cwd /tmp/here' }, ':let g:cmd = "xcwd "<CR>', { vars = { 'cmd' } })
  keycase('abbrev-both-modes', { 'abbrev ab abbreviation' }, 'iab <Esc>', { lines = { '' } })
  keycase('abbrev-after-delete', { 'iabbrev teh the', 'iunabbrev teh' }, 'iteh <Esc>', { lines = { '' } })
  keycase('abbrev-expr', { [[iabbrev <expr> dt "expanded"]] }, 'idt <Esc>', { lines = { '' } })
  keycase('abbrev-silent', { 'iabbrev <silent> teh the' }, 'iteh <Esc>', { lines = { '' } })
  keycase('abbrev-buffer', { 'iabbrev <buffer> teh the' }, 'iteh <Esc>', { lines = { '' } })
end)

-- =====================================================================
-- 5. keycodes.rs -- the key-name tables, both directions.
--
-- 187 entries in key_names_table, 44 in modifier_keys_table, 12 mouse
-- rows and a generated 430-line hash dispatch, and B13-4 replaces the
-- lot with a sorted const.  Every family gets a representative here,
-- and each is taken four ways: through nvim_replace_termcodes (the
-- name -> bytes direction), through keytrans (bytes -> name), through
-- Vimscript's "\<Key>" (a third entry point into the same tables) and
-- through a mapping's lhsraw/lhsrawalt (which is where a wrong K_SPECIAL
-- encoding shows up).
-- =====================================================================
local KEY_NAMES = {
  'Esc', 'CR', 'Return', 'Enter', 'NL', 'LF', 'Tab', 'Space', 'BS', 'Del',
  'Nul', 'Bslash', 'Bar', 'lt', 'Nop', 'Ignore', 'Undo', 'Help', 'Insert',
  'Up', 'Down', 'Left', 'Right', 'Home', 'End', 'PageUp', 'PageDown',
  'F1', 'F5', 'F12', 'F37', 'S-F1', 'xF1', 'xUp', 'xEnd',
  'k0', 'k9', 'kPlus', 'kMinus', 'kMultiply', 'kDivide', 'kEnter', 'kPoint',
  'kComma', 'kEqual', 'KP0', 'kUp', 'kHome',
  'C-A', 'C-a', 'C-@', 'C-^', 'C-\\', 'C-[', 'C-]', 'C-_', 'C-?',
  'M-a', 'A-a', 'M-A', 'D-a', 'T-a', 'S-a', 'C-S-a', 'M-C-a', 'C-M-S-a',
  'S-Left', 'C-Left', 'M-Left', 'S-Tab', 'C-Tab', 'S-Del',
  'LeftMouse', 'LeftDrag', 'LeftRelease', 'RightMouse', 'MiddleMouse',
  '2-LeftMouse', '3-LeftMouse', '4-LeftMouse', 'C-LeftMouse', 'S-LeftMouse',
  'ScrollWheelUp', 'ScrollWheelDown', 'ScrollWheelLeft', 'MouseMove',
  'X1Mouse', 'X2Mouse',
  'Plug', 'SNR', 'Cmd', 'ScriptCmd', 'CSI', 'xCSI', 'FocusGained',
  'FocusLost', 'Paste', 'PasteStart', 'PasteEnd',
  'char-97', 'char-0x41', 'Char-233', 'EOL', 'NotAKey', 'C-', '',
}

section('keycodes', function()
  for _, name in ipairs(KEY_NAMES) do
    local spelled = '<' .. name .. '>'
    local key = ('key %q'):format(spelled)
    local bytes = vim.api.nvim_replace_termcodes(spelled, true, true, true)
    local no_special = vim.api.nvim_replace_termcodes(spelled, true, true, false)
    local no_lt = vim.api.nvim_replace_termcodes(spelled, true, false, true)
    local from_part = vim.api.nvim_replace_termcodes(spelled, false, true, true)
    local trans = vcall('keytrans', bytes)
    local vimstr = vcall('eval', '"\\' .. spelled .. '"')
    struct(key, {
      termcodes = bytes,
      no_special = no_special,
      no_lt = no_lt,
      no_from_part = from_part,
      keytrans = trans,
      vimscript = vimstr,
    })
    emit(
      key,
      '| tc=' .. esc(bytes),
      '| nospec=' .. esc(no_special),
      '| nolt=' .. esc(no_lt),
      '| nofp=' .. esc(from_part),
      '| trans=' .. esc(tostring(trans)),
      '| vim=' .. esc(tostring(vimstr))
    )
  end
  -- lhsraw/lhsrawalt: the encoding a lhs is stored in.  A <C-x> lhs has
  -- two spellings (the K_SPECIAL form and the bare control byte) and
  -- only the dict shows the second one.
  for _, name in ipairs({
    '<C-a>', '<C-x><C-y>', '<M-a>', '<F5>', '<S-Left>', '<Up>', '<Tab>',
    '<Space>', '<lt>', '<Bar>', '<LeftMouse>', '<ScrollWheelUp>', '<Cmd>',
    '<C-@>', '<C-S-a>', '<k0>', '<kEnter>', '<Esc>x', '<Del>', '<BS>',
  }) do
    exec(('silent! nmap %s iM<Esc>'):format(name))
    answer(('lhs %q'):format(name), vcall('maparg', name, 'n', false, true))
  end
  run('keycodes listing', 'nmap')
  -- str2special's other entry point: the rhs side of a listing.
  for _, rhs in ipairs({ '<Esc>', '<C-a>', '<Space>', '<Bar>', '<lt>', '<Nop>', '<CR><Tab>' }) do
    exec(('silent! nmap ,zz %s'):format(rhs))
    run(('rhs %q'):format(rhs), 'nmap ,zz')
  end
  answer('keytrans empty', vcall('keytrans', ''))
  answer('keytrans plain', vcall('keytrans', 'abc'))
  answer('keytrans mixed', vcall('keytrans', tc('a<C-a>b<Esc><S-Left>c')))
  answer('keytrans high', vcall('keytrans', '\xc3\xa9'))
  answer('keytrans bad-args', vcall('keytrans'))
  answer('replace_termcodes empty', vim.api.nvim_replace_termcodes('', true, true, true))
  answer('replace_termcodes literal-lt', vim.api.nvim_replace_termcodes('a<b', true, true, true))
  answer('replace_termcodes backslash', vim.api.nvim_replace_termcodes('a\\<CR>b', true, true, true))
  answer('replace_termcodes csi', vim.api.nvim_replace_termcodes('\x80\xfdQ', true, true, true))
end)

-- =====================================================================
-- 6. getchar.rs -- getchar()/getcharstr()/getcharmod().
--
-- Fed from a typeahead queued with feedkeys(..., 'nt') in the same Lua
-- chunk: nothing is running the normal-mode loop here, so the queue
-- survives until getchar() drains it.  Every case queues at least as
-- many keys as it takes, because getchar() with an empty typeahead
-- blocks forever and there is nobody to type.
-- =====================================================================
section('getchar', function()
  local function queue(keys)
    pcall(vim.fn.feedkeys, tc(keys), 'nt')
  end
  local function take(label, fn, ...)
    local args = { ... }
    local res = vcall(fn, unpack(args))
    answer(label, res)
    return res
  end
  queue('ab<C-a>')
  take('getchar plain a', 'getchar')
  take('getchar plain b', 'getchar')
  take('getchar ctrl-a', 'getchar')
  take('getchar empty nonblock', 'getchar', 0)
  take('getchar empty peek', 'getchar', 1)
  queue('<S-Left>')
  take('getchar special', 'getchar')
  take('getcharmod after special', 'getcharmod')
  queue('<C-S-x>')
  take('getchar ctrl-shift', 'getchar')
  take('getcharmod ctrl-shift', 'getcharmod')
  queue('xyz')
  take('getcharstr x', 'getcharstr')
  take('getchar peek y', 'getchar', 1)
  take('getchar take y', 'getchar', 0)
  take('getcharstr z', 'getcharstr')
  queue('<Esc>')
  take('getchar esc', 'getchar')
  queue('\xc3\xa9')
  take('getchar utf8', 'getchar')
  take('getcharstr utf8 mod', 'getcharmod')
  queue('<LeftMouse>')
  take('getchar mouse', 'getchar')
  take('getcharmod mouse', 'getcharmod')
  answer('getmousepos', vcall('getmousepos'))
  queue('<F5>')
  take('getchar f5', 'getchar')
  queue('abc')
  take('getcharstr nonblock', 'getcharstr', 0)
  take('getcharstr peek', 'getcharstr', 1)
  take('getcharstr take', 'getcharstr', 0)
  take('getcharstr drain', 'getcharstr', 0)
  -- A key is queued first: if a bad {expr} is coerced to 0 rather than
  -- rejected, getchar() blocks, and there is nobody here to type.
  queue('Q')
  take('getchar bad-arg', 'getchar', 'x')
  take('getchar drain-after-bad', 'getchar', 0)
  answer('getcharsearch', vcall('getcharsearch'))
  answer('setcharsearch', vcall('setcharsearch', { char = 'x', forward = 1, ['until'] = 0 }))
  answer('getcharsearch after set', vcall('getcharsearch'))
  -- Mappings are *not* applied by getchar(); prove it rather than
  -- assume it, because the layering is exactly what B13-7 rewrites.
  exec('silent! nmap a bcd')
  queue('a')
  take('getchar with-map-pending', 'getchar')
end)

-- =====================================================================
-- 7. getchar.rs -- recording, replay, :normal and re-entry.
-- =====================================================================
section('record', function()
  keycase('record-simple', {}, 'qqiZZ<Esc>q', { regs = { 'q' } })
  keycase('record-and-replay', {}, 'qqiZ<Esc>q@q', { regs = { 'q' } })
  keycase('record-replay-twice', {}, 'qqiZ<Esc>q@q@@', { regs = { 'q' } })
  keycase('record-count-replay', {}, 'qqiZ<Esc>q3@q', { regs = { 'q' } })
  keycase('record-append', { 'let @q = "iA\\<Esc>"' }, 'qQiB<Esc>q', { regs = { 'q' } })
  keycase('record-uppercase-new', {}, 'qRiC<Esc>q', { regs = { 'r' } })
  keycase('record-into-numbered', {}, 'q1iZ<Esc>q', { regs = { '1' } })
  keycase('record-nested-q', {}, 'qqiZ<Esc>qq', { regs = { 'q' } })
  keycase('record-with-map', { 'nmap ,z iM<Esc>' }, 'qq,zq@q', { regs = { 'q' } })
  keycase('replay-missing-reg', {}, '@z', { via = 'normal' })
  keycase('replay-colon-reg', { 'let @: = "let g:cmd = 5"' }, '@:', { vars = { 'cmd' }, via = 'normal' })
  keycase('normal-with-map', { 'nmap ,z iM<Esc>' }, ',z', { via = 'normal' })
  keycase('normal-bang-ignores-map', { 'nmap ,z iM<Esc>' }, ',z', { via = 'normal!' })
  keycase('normal-incomplete', {}, 'i', { via = 'normal' })
  keycase('normal-count', {}, '3x', { via = 'normal' })
  keycase('ctrl-o-normal', {}, 'iAB<C-o>0X<Esc>', { lines = { '' } })
  keycase('ctrl-bsl-ctrl-o', { 'inoremap <C-l> <C-\\><C-o>dd' }, 'iAB<C-l>', { lines = { 'one', 'two' } })
  keycase('ctrl-o-with-map', { 'nmap ,z x' }, 'iAB<C-o>,z<Esc>', { lines = { '' } })
  keycase('insert-ctrl-r', { 'let @a = "PASTED"' }, 'i<C-r>a<Esc>', { lines = { '' } })
  keycase('insert-ctrl-v', {}, 'i<C-v>065<Esc>', { lines = { '' } })
  keycase('insert-ctrl-k', {}, 'i<C-k>a:<Esc>', { lines = { '' } })
  keycase('redo-dot', { 'nmap ,z x' }, ',z.', { lines = { 'abcdef' } })
  keycase('undo-after-map', { 'nmap ,z x' }, ',z,zu', { lines = { 'abcdef' } })
end)

-- =====================================================================
-- 8. getchar.rs -- feedkeys()'s mode letters.
--
-- 'n' 't' 'x' 'i' '!' 'm' 'L' and the combinations that matter.  Cases
-- without 'x' leave the keys queued, so each is followed by an explicit
-- drain and the state is taken after both.
-- =====================================================================
section('feedkeys-modes', function()
  local function fkcase(label, prep, keys, mode, opts)
    opts = opts or {}
    clear_maps()
    exec('silent! enew!')
    pcall(vim.api.nvim_buf_set_lines, 0, 0, -1, false, opts.lines or { 'abcdef' })
    pcall(vim.api.nvim_win_set_cursor, 0, { 1, 0 })
    for _, cmd in ipairs(prep or {}) do
      exec('silent! ' .. cmd)
    end
    vim.v.errmsg = ''
    local ok, err = pcall(vim.fn.feedkeys, tc(keys), mode)
    local before = vim.api.nvim_buf_get_lines(0, 0, -1, false)
    local editor_mode = vim.api.nvim_get_mode().mode
    local queued = vim.fn.state()
    -- Drain whatever is left, so the next case starts clean and so the
    -- difference between "ran" and "queued" is visible.
    pcall(vim.fn.feedkeys, '', 'x')
    pcall(vim.cmd, 'silent! stopinsert')
    pcall(vim.fn.feedkeys, tc('<Esc>'), 'xt')
    local after = vim.api.nvim_buf_get_lines(0, 0, -1, false)
    local state = {
      keys = keys,
      mode = mode,
      before = before,
      after = after,
      editor_mode = editor_mode,
      pending = queued,
      err = ok and '' or scrub(tostring(err)),
      errmsg = vim.v.errmsg,
    }
    struct('feedkeys ' .. label, state)
    emit(
      'feedkeys ' .. label,
      '| mode=' .. esc(mode),
      '| keys=' .. esc(keys),
      '| before=' .. esc(vim.inspect(before):gsub('%s+', ' ')),
      '| after=' .. esc(vim.inspect(after):gsub('%s+', ' ')),
      '| emode=' .. esc(editor_mode),
      '| pending=' .. esc(queued)
    )
  end
  fkcase('x-plain', {}, 'x', 'x')
  fkcase('n-noremap', { 'nmap x iA<Esc>' }, 'x', 'nx')
  fkcase('m-remap', { 'nmap x iA<Esc>' }, 'x', 'mx')
  fkcase('t-as-typed', { 'nmap x iA<Esc>' }, 'x', 'xt')
  fkcase('no-x-queued', {}, 'x', 't')
  fkcase('i-insert-at-front', {}, 'x', 'i')
  fkcase('i-then-x', {}, 'xx', 'ix')
  -- NO '!' ARM RUNS IN-PROCESS.  '!' means "do not end Insert mode",
  -- so the drain that follows asks the *real* input stream for the next
  -- key -- and in `-l` script mode that stream is not something a
  -- scenario controls.  Measured both ways: with stdin on a terminal
  -- nvim exits(0) mid-script and silently truncates the report; with
  -- stdin at EOF it spins on the empty read until the harness timeout
  -- (180 s of CPU).  The '!' arms live in section 12, against the
  -- child, which has a main loop and an input stream we type at.  'L'
  -- is out for the same reason and then some: lowlevel keys are put on
  -- the *input* stream rather than the typeahead, so 'Lx' waits on a
  -- main loop that `-l` script mode does not run.
  fkcase('empty-keys', {}, '', 'x')
  fkcase('empty-mode', {}, 'x', '')
  fkcase('bad-mode-letter', {}, 'x', 'Q')
  fkcase('special-keys', {}, '<Esc>ihi<Esc>', 'xt', { lines = { '' } })
  fkcase('escape-csi', {}, '\x80\xfdQ', 'x')
  fkcase('nested-feedkeys', { 'nnoremap ,f :call feedkeys("iN\\<Esc>", "x")<CR>' }, ',f', 'xt')
  answer('feedkeys bad-args', vcall('feedkeys'))
  answer('feedkeys nonstring', vcall('feedkeys', 1))
end)

-- =====================================================================
-- 9-10. Reserved, and left reserved.  B13-3's cmdline/completion and
-- message sections (15-26) are appended at the *bottom* of this file
-- instead: `lnum` is one of mapblock_fill_dict's twenty keys and every
-- mapping above carries the line it was defined on, so an insertion
-- here would re-baseline the whole `.struct` artifact for nothing.
-- =====================================================================

-- =====================================================================
-- 11-12. The child.  See the header: the timeout matrix cannot be
-- driven in-process, because feedkeys(..., 'x') runs with
-- `ex_normal_busy` set and `vgetorpeek` then refuses to wait.
-- =====================================================================
local child_stderr = {}
local chan = nil

local function child_start()
  child_stderr[#child_stderr + 1] = '--- child nvim start ---'
  local ok, res = pcall(vim.fn.jobstart, {
    vim.v.progpath, '--embed', '--headless', '-u', 'NONE', '-i', 'NONE',
  }, {
    rpc = true,
    on_stderr = function(_, data)
      for _, line in ipairs(data or {}) do
        if line ~= '' then
          child_stderr[#child_stderr + 1] = scrub(line)
        end
      end
    end,
  })
  chan = ok and res or nil
  return chan ~= nil and chan > 0
end

local function creq(method, ...)
  if not chan then
    return 'NO-CHILD'
  end
  local ok, res = pcall(vim.rpcrequest, chan, method, ...)
  if ok then
    return res
  end
  return 'ERROR ' .. scrub(tostring(res))
end

--- Fire and forget.  Anything that can leave the child inside its key
--- loop -- every `feedkeys()` with 'x', every '!' arm -- has to be a
--- notification: an rpc*request* for a call that does not return until
--- the keys are consumed deadlocks the parent, and the parent is the
--- process holding the report.
local function cnotify(method, ...)
  if chan then
    pcall(vim.rpcnotify, chan, method, ...)
  end
end

local function cwait(ms)
  vim.wait(ms, function()
    return false
  end, 10)
end

--- The only probe that is safe against a child sitting inside
--- vgetorpeek: nvim_get_mode is `fast`, everything else is queued to a
--- main loop that is not running.
local function cblocking()
  local mode = creq('nvim_get_mode')
  if type(mode) ~= 'table' then
    return nil, tostring(mode)
  end
  return mode.blocking and true or false, mode.mode
end

local CHILD_SNAP = [[
local m = vim.api.nvim_get_mode()
local ok, cur = pcall(vim.api.nvim_win_get_cursor, 0)
return {
  lines = vim.api.nvim_buf_get_lines(0, 0, -1, false),
  cursor = ok and { cur[1], cur[2] + 1 } or { 0, 0 },
  mode = m.mode,
  blocking = m.blocking,
  errmsg = vim.v.errmsg,
  recording = vim.fn.reg_recording(),
  state = vim.fn.state(),
  reg_q = vim.fn.getreg('q'),
  cmd = vim.g.cmd == nil and '' or tostring(vim.g.cmd),
}
]]

--- Get the child back to a state where a non-fast request will be
--- answered.  A child that cannot be unblocked is replaced: the point
--- is that a hang becomes a recorded difference, not a wedged sweep.
local function cunwedge()
  for _ = 1, 6 do
    local blocked = cblocking()
    if blocked == false then
      return true
    end
    if blocked == nil then
      break
    end
    creq('nvim_input', '<Esc>')
    cwait(80)
  end
  pcall(vim.fn.jobstop, chan)
  chan = nil
  child_start()
  cwait(200)
  return false
end

local function csnap(label, extra)
  local blocked, mode = cblocking()
  if blocked ~= false then
    local recovered = cunwedge()
    local state = vim.tbl_extend('force', {
      WEDGED = true,
      blocking_mode = mode,
      recovered = recovered,
    }, extra or {})
    struct(label, state)
    emit(label, '| WEDGED blocking_mode=' .. esc(tostring(mode)) .. ' recovered=' .. tostring(recovered))
    return state
  end
  local snap = creq('nvim_exec_lua', CHILD_SNAP, {})
  if type(snap) ~= 'table' then
    snap = { ERROR = tostring(snap) }
  end
  for key, value in pairs(extra or {}) do
    snap[key] = value
  end
  struct(label, snap)
  emit(
    label,
    '|',
    esc(vim.inspect(snap.lines or {}):gsub('%s+', ' ')),
    '| cur=' .. (snap.cursor and (snap.cursor[1] .. ',' .. snap.cursor[2]) or '?'),
    '| mode=' .. esc(tostring(snap.mode)),
    '| blocking=' .. tostring(snap.blocking),
    '| state=' .. esc(tostring(snap.state)),
    '| err=' .. esc(tostring(snap.errmsg))
  )
  return snap
end

local function cprep(cmds)
  cunwedge()
  -- Get out of whatever mode the previous case ended in *before* the
  -- first non-fast request.  Without this a case that left the child in
  -- Insert mode has the next case's keys typed into that buffer, and
  -- the answer becomes a function of the order the cases are written
  -- in (measured: three cases in a row read as literal text).
  creq('nvim_input', '<Esc>')
  cwait(60)
  cunwedge()
  creq('nvim_command', 'silent! stopinsert')
  creq('nvim_command', 'silent! enew!')
  for _, prefix in ipairs({ '', 'n', 'v', 'x', 's', 'o', 'i', 'l', 'c', 't' }) do
    creq('nvim_command', 'silent! ' .. prefix .. 'mapclear')
  end
  creq('nvim_command', 'silent! mapclear!')
  creq('nvim_command', 'silent! abclear')
  creq('nvim_command', 'silent! set timeout ttimeout timeoutlen=1000 ttimeoutlen=50')
  creq('nvim_command', 'silent! set maxmapdepth=1000 report=9999 nomore shortmess=filnxtToOF')
  creq('nvim_command', 'silent! set noswapfile noshowcmd noshowmode langmap= nolangremap')
  creq('nvim_command', 'silent! unlet! g:cmd')
  creq('nvim_command', 'silent! let v:errmsg = ""')
  creq('nvim_command', 'silent! call setreg("q", "")')
  for _, cmd in ipairs(cmds or {}) do
    creq('nvim_command', 'silent! ' .. cmd)
  end
end

-- `steps` is a list of { input = <keys>, wait = <ms>, probe = <bool> }.
-- A `probe` step records nvim_get_mode() *while the case is mid-flight*
-- -- that is the timeout assertion: "with 'notimeout' and no further
-- input the child is still waiting" is a fact about vgetorpeek, and it
-- is recorded rather than raced.
local function tcase(label, prep, steps)
  cprep(prep)
  local trace = {}
  for _, step in ipairs(steps) do
    if step.input then
      creq('nvim_input', step.input)
    end
    if step.wait then
      cwait(step.wait)
    end
    if step.probe then
      local blocked, mode = cblocking()
      trace[#trace + 1] = ('after %q blocking=%s mode=%s'):format(
        step.input or '',
        tostring(blocked),
        tostring(mode)
      )
    end
  end
  csnap(label, { trace = trace })
  if #trace > 0 then
    for _, line in ipairs(trace) do
      emit(label, '  ~', esc(line))
    end
  end
end

section('typeahead-timeout', function()
  if not chan and not child_start() then
    emit('typeahead-timeout !! child nvim would not start')
    return
  end
  cwait(200)
  local AMBIG = { 'nnoremap ,a iA<Esc>', 'nnoremap ,ab iB<Esc>' }
  -- The heart of it: a prefix that is both a complete mapping and the
  -- start of a longer one.  What resolves it is 'timeout'.
  tcase('timeout-on-short-len', vim.list_extend({ 'set timeout timeoutlen=30' }, AMBIG), {
    { input = ',a', wait = 250, probe = true },
    { input = 'b', wait = 250 },
  })
  tcase('timeout-on-long-len', vim.list_extend({ 'set timeout timeoutlen=4000' }, AMBIG), {
    { input = ',a', wait = 120, probe = true },
    { input = 'b', wait = 250 },
  })
  tcase('timeout-off-waits', vim.list_extend({ 'set notimeout' }, AMBIG), {
    { input = ',a', wait = 300, probe = true },
    { input = 'b', wait = 250 },
  })
  tcase('timeout-off-then-esc', vim.list_extend({ 'set notimeout' }, AMBIG), {
    { input = ',a', wait = 300, probe = true },
    { input = '<Esc>', wait = 250 },
  })
  tcase('timeout-on-no-more-input', vim.list_extend({ 'set timeout timeoutlen=30' }, AMBIG), {
    { input = ',a', wait = 400, probe = true },
  })
  tcase('timeout-complete-long', vim.list_extend({ 'set timeout timeoutlen=30' }, AMBIG), {
    { input = ',ab', wait = 250, probe = true },
  })
  tcase('timeout-nowait-beats-len', {
    'set timeout timeoutlen=4000',
    'nnoremap <nowait> ,a iA<Esc>',
    'nnoremap ,ab iB<Esc>',
  }, {
    { input = ',a', wait = 250, probe = true },
    { input = 'b', wait = 250 },
  })
  -- 'ttimeout'/'ttimeoutlen' govern the *key code* half: a lhs that
  -- starts with ESC is indistinguishable from the start of a terminal
  -- escape sequence, and that is the branch these two options pick.
  local ESCMAP = { 'nnoremap <Esc>a iE<Esc>', 'nnoremap <Esc> iX<Esc>' }
  tcase('ttimeout-on-short-len', vim.list_extend({ 'set timeout timeoutlen=4000 ttimeout ttimeoutlen=30' }, ESCMAP), {
    { input = '<Esc>', wait = 250, probe = true },
    { input = 'a', wait = 250 },
  })
  tcase('ttimeout-on-long-len', vim.list_extend({ 'set timeout timeoutlen=4000 ttimeout ttimeoutlen=4000' }, ESCMAP), {
    { input = '<Esc>', wait = 150, probe = true },
    { input = 'a', wait = 250 },
  })
  tcase('ttimeout-off', vim.list_extend({ 'set notimeout nottimeout ttimeoutlen=30' }, ESCMAP), {
    { input = '<Esc>', wait = 250, probe = true },
    { input = 'a', wait = 250 },
  })
  tcase('ttimeoutlen-negative', vim.list_extend({ 'set timeout timeoutlen=60 ttimeout ttimeoutlen=-1' }, ESCMAP), {
    { input = '<Esc>', wait = 300, probe = true },
    { input = 'a', wait = 250 },
  })
  -- A three-way prefix chain: two of the three are complete mappings.
  tcase('timeout-three-way', {
    'set timeout timeoutlen=30',
    'nnoremap ,a iA<Esc>',
    'nnoremap ,ab iB<Esc>',
    'nnoremap ,abc iC<Esc>',
  }, {
    { input = ',ab', wait = 250, probe = true },
    { input = 'c', wait = 250 },
  })
  tcase('timeout-three-way-fast', {
    'set timeout timeoutlen=4000',
    'nnoremap ,a iA<Esc>',
    'nnoremap ,ab iB<Esc>',
    'nnoremap ,abc iC<Esc>',
  }, {
    { input = ',ab', wait = 100, probe = true },
    { input = 'c', wait = 250 },
  })
  -- An incomplete *builtin* (not a mapping) also parks vgetorpeek, and
  -- 'timeout' does not apply to it.
  tcase('incomplete-operator', { 'set timeout timeoutlen=30' }, {
    { input = 'd', wait = 250, probe = true },
    { input = 'w', wait = 250 },
  })
  tcase('incomplete-replace', { 'set timeout timeoutlen=30' }, {
    { input = 'r', wait = 250, probe = true },
    { input = 'z', wait = 250 },
  })
end)

section('typeahead-input', function()
  if not chan and not child_start() then
    emit('typeahead-input !! child nvim would not start')
    return
  end
  -- nvim_input takes key *notation*; nvim_feedkeys takes bytes.  Both
  -- ends of nvim_replace_termcodes are exercised by feeding the same
  -- sequence through each.
  tcase('input-notation', {}, { { input = 'iabc<Esc>', wait = 200 } })
  tcase('input-raw-esc', {}, { { input = 'iabc\27', wait = 200 } })
  tcase('input-with-map', { 'nnoremap ,z iM<Esc>' }, { { input = ',z', wait = 200 } })
  tcase('input-count', {}, { { input = '3ix<Esc>', wait = 300 } })
  tcase('input-piecemeal', { 'nnoremap ,z iM<Esc>' }, {
    { input = ',', wait = 150, probe = true },
    { input = 'z', wait = 200 },
  })
  cprep({})
  cnotify('nvim_call_function', 'feedkeys', { creq('nvim_replace_termcodes', 'iFED<Esc>', true, true, true), 'nt' })
  cwait(250)
  csnap('feedkeys-via-rpc')
  cprep({ 'nnoremap ,z iM<Esc>' })
  cnotify('nvim_call_function', 'feedkeys', { ',z', 'nt' })
  cwait(250)
  csnap('feedkeys-rpc-noremap')
  cprep({ 'nnoremap ,z iM<Esc>' })
  cnotify('nvim_call_function', 'feedkeys', { ',z', 'mt' })
  cwait(250)
  csnap('feedkeys-rpc-remap')
  -- The '!' arms, which cannot be run in-process (see section 8).
  for _, arm in ipairs({
    { 'bang-x', 'iQ', '!x' },
    { 'bang-only', 'iQ', '!' },
    { 'x-ends-insert', 'iQ', 'x' },
    { 'i-front', 'iR', 'i' },
    -- 'L' alone and 'Lt' are here; 'Lx' is NOT.  `feedkeys(k, 'Lx')`
    -- wedges the process outright -- 100% CPU and even the `fast`
    -- nvim_get_mode goes unanswered, so the harness cannot recover and
    -- cannot record anything but the harness timeout.  Recorded in
    -- docket.md; it needs an upstream-C comparison, not a sweep case.
    { 'L-lowlevel', 'x', 'L' },
    { 'L-typed', 'x', 'Lt' },
    { 'n-noremap', 'x', 'nx' },
    { 'm-remap', 'x', 'mx' },
  }) do
    cprep({ 'nnoremap x iA<Esc>' })
    creq('nvim_command', 'silent! call setline(1, "abcdef")')
    local bytes = creq('nvim_replace_termcodes', arm[2], true, true, true)
    cnotify('nvim_call_function', 'feedkeys', { bytes, arm[3] })
    cwait(250)
    csnap('feedkeys-mode-' .. arm[1], { fk_mode = arm[3], fk_keys = arm[2] })
    creq('nvim_input', '<Esc>')
    cwait(60)
  end
  -- getchar() against a real input stream: the child blocks in
  -- safe_vgetc until something is typed, which is the layering
  -- in-process cases cannot reach.
  cprep({})
  creq('nvim_command', 'silent! call timer_start(10, {-> execute("let g:cmd = 1")})')
  local blocked_before
  do
    creq('nvim_command', 'silent! let g:cmd = 0')
    -- Ask for a character, then type one.  nvim_input is `fast`, so it
    -- is delivered while the child sits in getchar().
    vim.rpcnotify(chan, 'nvim_command', 'silent! let g:got = getchar()')
    cwait(200)
    blocked_before = select(1, cblocking())
    creq('nvim_input', 'K')
    cwait(250)
  end
  csnap('getchar-real-input', { blocked_before = blocked_before })
  answer('getchar-real-value', creq('nvim_eval', 'get(g:, "got", -1)'))
  -- Recording and replay through the real key loop.
  tcase('record-real', {}, { { input = 'qqiZ<Esc>q', wait = 300 } })
  tcase('record-replay-real', {}, { { input = 'qqiZ<Esc>q@q', wait = 400 } })
  tcase('mapping-in-recording', { 'nnoremap ,z iM<Esc>' }, { { input = 'qq,zq@q', wait = 400 } })
  -- v:count and <C-o> through the real loop.
  tcase('count-through-map-real', { 'nnoremap ,c <Cmd>let g:cmd = v:count<CR>' }, {
    { input = '7,c', wait = 300 },
  })
  -- The `:`-rhs form of the same thing, where the count becomes a
  -- *range* rather than v:count and the command rejects it.
  tcase('count-through-colon-map', { 'nnoremap ,c :let g:cmd = v:count<CR>' }, {
    { input = '7,c', wait = 300 },
  })
  tcase('ctrl-o-real', {}, { { input = 'iAB<C-o>0X<Esc>', wait = 300 } })
  tcase('cmdline-map-real', { 'cnoremap ,q let g:cmd = 9' }, { { input = ':,q<CR>', wait = 300 } })
  tcase('abbrev-real', { 'iabbrev teh the' }, { { input = 'iteh <Esc>', wait = 300 } })
  tcase('expr-map-real', { [[nnoremap <expr> ,e '"iX\<Esc>"']] }, { { input = ',e', wait = 300 } })
  tcase('recursive-real', { 'set maxmapdepth=20', 'nmap ,r ,s', 'nmap ,s ,r' }, {
    { input = ',r', wait = 400, probe = true },
  })
end)


-- =====================================================================
-- 15-26 (B13-3).  cmdline, completion and messages -- ex_getln.rs,
-- cmdexpand.rs and message.rs.
--
-- APPENDED, deliberately, after everything above.  `lnum` is one of
-- mapblock_fill_dict's twenty keys and every mapping this script defines
-- carries the line it was defined on, so an insertion anywhere above
-- section 8 re-baselines the whole `.struct` artifact for no reason.
-- The child from sections 11-12 is still alive at this point and these
-- sections may use it; the teardown stays at the bottom of the file.
--
-- Everything here runs in-process.  That is a measured choice, not an
-- assumption: `command_line_enter`, `do_more_prompt`, `wait_return`,
-- `input()`, `inputlist()`, `confirm()` and the wildmenu key loop all
-- run their *own* `vgetc`, and a `feedkeys(..., 't')` issued *before*
-- the call queues the answers where that inner loop finds them -- the
-- `popupprobe.lua` recipe from B12-7, which works here too (measured:
-- an 80-message `:messages` under `'more'` is walked and dismissed with
-- pre-queued keys, and `input()` returns the queued line).  What must
-- never happen is a case whose queued keys run out: script mode then
-- asks the real input stream, and at EOF nvim spins at 100% CPU
-- (docket D-B13-2).  Every case below terminates its own prompt.
-- =====================================================================

-- Four answers below are not run-stable, and all four were found by the
-- three-consecutive-runs rule rather than by reading the code: `:let`
-- with no argument dumps `v:starttime` and `v:servername`, a Lua
-- traceback carries an ASLR'd `0x...` frame, and `v:argv`/`v:progpath`
-- name the binary under test (which is the whole point of an A/B, so
-- the two sides would differ on path alone).
--
-- The mask is installed HERE rather than in `scrub` itself: sections
-- 1-14 have already run by the time this line executes, so their
-- artifacts stay byte-identical to the B13-2 baseline and the
-- re-baseline diff is exactly the appended sections.  `scrub` is a
-- local of the main chunk, and every earlier closure captured the
-- variable, not the value -- so this widens it from here on and only
-- from here on.
local scrub_prefix = scrub
scrub = function(text)
  text = scrub_prefix(text)
  text = text:gsub(vim.pesc(vim.v.progpath), '<NVIM>')
  text = text:gsub('0x%x+', '<ADDR>')
  text = text:gsub('/tmp/nvim%.[^/\'"%s]+/[^/\'"%s]+/nvim%.%d+%.%d+', '<SERVER>')
  text = text:gsub('(v:starttime%s+#)%d+', '%1<TIME>')
  text = text:gsub('(<SCRIPT>:)%d+', '%1<LINE>')
  -- `:command`'s "Last set from" line names a Lua-defined command's
  -- source as a *relative* path resolved against the cwd nvim STARTED
  -- in, not against $VIMRUNTIME -- so the same binary answers
  -- differently depending on where the sweep was invoked from (found by
  -- `keyverify` running from the repo root against a baseline taken
  -- from a scratch directory; the three-runs rule cannot see it,
  -- because all three runs share a cwd).
  text = text:gsub('[^%s\'"]*vim/_core/', '<RTLUA>/')
  return text
end

--- A fixture directory for file/dir completion.  Fixed names, fixed
--- order once sorted; `getcompletion()` answers absolute-ish paths
--- relative to the cwd, which `reset()` pins to <WORK>.
local function cmdfixture()
  vim.fn.mkdir(work .. '/cdir/sub', 'p')
  -- `Alpha*` exists so that two candidates differ *only* in case at the
  -- point where they diverge.  Without a pair like that,
  -- `find_longest_match`'s 'wildignorecase' fold cannot change an answer
  -- and a mutation on it is invisible (it was: cx-longest-ignorecase,
  -- NOT CAUGHT on the first corpus).
  for _, name in ipairs({
    'cdir/alpha.txt', 'cdir/alpha.vim', 'cdir/beta.txt', 'cdir/gamma.log',
    'cdir/sub/deep.txt', 'cdir/Alphabet.txt', 'cdir/ALPHAGO.txt',
  }) do
    local fd = io.open(work .. '/' .. name, 'w')
    if fd then
      fd:write('x\n')
      fd:close()
    end
  end
end

-- ---------------------------------------------------------------------
-- The cmdline sampler.  Autocmds are the only way to see the command
-- line *while it is being edited*: every getcmd* function answers
-- NUL/"" once `command_line_enter` has returned.  Each fired event
-- records the eight getcmd* answers plus `cmdcomplete_info()` and
-- `v:event`, so one scenario yields a trace rather than an endpoint.
-- ---------------------------------------------------------------------
local CMD_EVENTS = {
  'CmdlineEnter', 'CmdlineChanged', 'CmdlineLeave',
  'CmdwinEnter', 'CmdwinLeave',
}
local CMD_GETTERS = {
  'getcmdline', 'getcmdtype', 'getcmdpos', 'getcmdscreenpos',
  'getcmdcompltype', 'getcmdcomplpat', 'getcmdprompt', 'getcmdwintype',
}

local cmdlog = {}
local cmdgroup = nil
-- Set by the one section that prints the per-keystroke trace.
local cmdtrace = false

local function cmdwatch(trace)
  cmdtrace = trace and true or false
  cmdgroup = vim.api.nvim_create_augroup('keysweepCmdline', { clear = true })
  for _, event in ipairs(CMD_EVENTS) do
    vim.api.nvim_create_autocmd(event, {
      group = cmdgroup,
      pattern = '*',
      callback = function(args)
        local sample = { event = event, afile = scrub(args.file or '') }
        for _, getter in ipairs(CMD_GETTERS) do
          sample[getter] = vcall(getter)
        end
        sample.cmdcomplete_info = vcall('cmdcomplete_info')
        local ok, event_dict = pcall(vim.deepcopy, vim.v.event)
        sample.v_event = ok and event_dict or 'ERROR'
        cmdlog[#cmdlog + 1] = sample
      end,
    })
  end
end

local function cmdunwatch()
  if cmdgroup then
    pcall(vim.api.nvim_del_augroup_by_id, cmdgroup)
    cmdgroup = nil
  end
end

--- One command-line scenario: prep, feed, record the whole event trace
--- plus what the buffer and the globals look like afterwards.
local function cmdcase(label, prep, keys, opts)
  opts = opts or {}
  clear_maps()
  exec('silent! enew!')
  pcall(vim.api.nvim_buf_set_lines, 0, 0, -1, false, opts.lines or DEFAULT_LINES)
  pcall(vim.api.nvim_win_set_cursor, 0, opts.cursor or { 1, 0 })
  for _, cmd in ipairs(prep or {}) do
    exec('silent! ' .. cmd)
  end
  cmdlog = {}
  vim.v.errmsg = ''
  pcall(vim.api.nvim_set_var, 'q', '')
  -- Anything the case wants answered by an inner vgetc has to be in the
  -- typeahead before the outer feedkeys runs.
  if opts.pre then
    pcall(vim.fn.feedkeys, tc(opts.pre), 't')
  end
  local ok, err = pcall(vim.fn.feedkeys, tc(keys), opts.mode or 'xt')
  local state = {
    keys = keys,
    lines = vim.api.nvim_buf_get_lines(0, 0, -1, false),
    mode = vim.api.nvim_get_mode().mode,
    errmsg = vim.v.errmsg,
    err = ok and '' or scrub(tostring(err):gsub('.-:%s*Vim', 'Vim')),
    g_q = vim.g.q == nil and vim.NIL or vim.g.q,
    events = cmdlog,
    histlast = vcall('histget', opts.hist or ':', -1),
    histnr = vcall('histnr', opts.hist or ':'),
  }
  local okc, cur = pcall(vim.api.nvim_win_get_cursor, 0)
  state.cursor = okc and { cur[1], cur[2] + 1 } or { 0, 0 }
  struct(label, state)
  emit(
    label,
    '>',
    esc(keys),
    '| n=' .. #cmdlog,
    '|',
    esc(vim.inspect(state.lines):gsub('%s+', ' ')),
    '| q=' .. esc(vim.inspect(state.g_q):gsub('%s+', ' ')),
    '| mode=' .. esc(state.mode),
    '| hist=' .. esc(tostring(state.histlast)),
    '| err=' .. esc(state.errmsg ~= '' and state.errmsg or state.err)
  )
  -- The per-keystroke trace is verbose (one line per CmdlineChanged),
  -- so only the section built around it prints it.  Everything is in
  -- the `.struct` line either way -- that artifact is the byte oracle,
  -- the report is the readable view.
  for index, sample in ipairs(cmdtrace and cmdlog or {}) do
    emit(
      label,
      '  ~' .. index,
      sample.event,
      'line=' .. esc(tostring(sample.getcmdline)),
      'type=' .. esc(tostring(sample.getcmdtype)),
      'pos=' .. tostring(sample.getcmdpos),
      'scr=' .. tostring(sample.getcmdscreenpos),
      'ct=' .. esc(tostring(sample.getcmdcompltype)),
      'cp=' .. esc(tostring(sample.getcmdcomplpat)),
      'prompt=' .. esc(tostring(sample.getcmdprompt)),
      'win=' .. esc(tostring(sample.getcmdwintype)),
      'ci=' .. esc(vim.inspect(sample.cmdcomplete_info):gsub('%s+', ' '))
    )
  end
  cmdlog = {}
  pcall(vim.cmd, 'silent! stopinsert')
  feed('<Esc>')
end

-- =====================================================================
-- 15. ex_getln.rs -- the command line as a state machine.
--
-- `command_line_enter` is 557 lines and `command_line_handle_key` 412;
-- between them they own every answer below.  The trace is per-keystroke
-- on purpose: a rewrite that gets the *end* state right and the
-- intermediate `getcmdpos` wrong is the failure mode a final-state-only
-- oracle cannot see, and `getcmdcompltype`/`getcmdcomplpat` are the only
-- in-flight view of `set_one_cmd_context`'s answer.
-- =====================================================================
section('cmdline-events', function()
  cmdfixture()
  cmdwatch(true)
  cmdcase('cmd-colon', {}, ':let g:q = 5<CR>')
  cmdcase('cmd-colon-abort', {}, ':let g:q = 6<Esc>')
  cmdcase('cmd-colon-ctrl-c', {}, ':let g:q = 7<C-c>')
  cmdcase('cmd-search-fwd', {}, '/second<CR>')
  cmdcase('cmd-search-back', {}, '?alpha<CR>')
  cmdcase('cmd-search-offset', {}, '/second/e<CR>')
  cmdcase('cmd-search-nomatch', {}, '/nosuchtext<CR>')
  cmdcase('cmd-filter', {}, ':.!tr a-z A-Z<CR>')
  cmdcase('cmd-range', {}, ':1,2s/a/Z/g<CR>')
  cmdcase('cmd-empty', {}, ':<CR>')
  cmdcase('cmd-bar', {}, ':let g:q = 1 | let g:q = 2<CR>')
  -- Nested command lines: `<C-r>=` and `:` inside `input()` both raise
  -- `cmdlevel`, which is the field the ext_cmdline protocol carries.
  cmdcase('cmd-ctrl-r-eq', {}, ':let g:q = <C-r>=1+2<CR><CR>')
  cmdcase('cmd-ctrl-r-eq-err', {}, ':let g:q = <C-r>=nosuchfn()<CR><CR><CR>')
  cmdcase('cmd-ctrl-r-reg', { 'call setreg("a", "REGA")' }, ':let g:q = "<C-r>a"<CR>')
  cmdcase('cmd-ctrl-r-ctrl-w', {}, ':let g:q = "<C-r><C-w>"<CR>')
  cmdcase('cmd-ctrl-r-ctrl-a', {}, ':let g:q = "<C-r><C-a>"<CR>')
  cmdcase('cmd-ctrl-r-ctrl-l', {}, ':let g:q = "<C-r><C-l>"<CR>')
  cmdcase('cmd-ctrl-r-ctrl-f', { 'edit ' .. work .. '/one.txt' }, ':let g:q = "<C-r><C-f>"<CR>')
  cmdcase('cmd-ctrl-r-ctrl-p', { 'edit ' .. work .. '/one.txt' }, ':let g:q = "<C-r><C-p>"<CR>')
  -- The completion contexts, sampled through getcmdcompltype rather
  -- than getcompletion(): this is the in-flight answer, and it is what
  -- ext_cmdline and the wildmenu both read.
  for _, probe in ipairs({
    { 'ct-command', ':se' },
    { 'ct-option', ':set sh' },
    { 'ct-optval', ':set backspace=' },
    { 'ct-file', ':edit cdir/a' },
    { 'ct-buffer', ':buffer ' },
    { 'ct-help', ':help getcmdl' },
    { 'ct-highlight', ':highlight Nor' },
    { 'ct-mapping', ':nmap ,' },
    { 'ct-augroup', ':augroup ' },
    { 'ct-event', ':autocmd Buf' },
    { 'ct-var', ':let g:' },
    { 'ct-expr', ':echo g:' },
    { 'ct-function', ':call getcm' },
    { 'ct-syntax', ':syntax ' },
    { 'ct-user', ':command Fo' },
    { 'ct-sign', ':sign ' },
    { 'ct-menu', ':menu ' },
    { 'ct-history', ':history ' },
    { 'ct-shellcmd', ':!l' },
    { 'ct-env', ':echo $HO' },
    { 'ct-lua', ':lua vim.ap' },
    { 'ct-messages', ':messages ' },
    { 'ct-substitute', ':s/a/b/' },
    { 'ct-global', ':g/a/d' },
  }) do
    cmdcase(probe[1], {}, probe[2] .. '<Esc>')
  end
  cmdunwatch()
end)

-- =====================================================================
-- 16. ex_getln.rs -- the editing keys.
--
-- Each is one arm of `command_line_handle_key`.  The answer recorded is
-- what the line looked like when it was accepted, taken from the
-- CmdlineLeave sample, so a wrong cursor motion shows up as a wrong
-- string rather than as nothing at all.
-- =====================================================================
section('cmdline-edit', function()
  cmdfixture()
  cmdwatch()
  local function edit(label, keys, prep)
    cmdcase(label, prep or {}, ':let g:q = "abcdef"' .. keys .. '<CR>')
  end
  edit('edit-plain', '')
  edit('edit-bs', '<BS><BS>"')
  edit('edit-ctrl-h', '<C-h><C-h>"')
  edit('edit-ctrl-w', '<C-w>Z"')
  edit('edit-ctrl-u', '<C-u>let g:q = "cleared"')
  edit('edit-ctrl-b-e', '<C-b>X<C-e>Y')
  edit('edit-left-right', '<Left><Left>Z<Right>Y')
  edit('edit-home-end', '<Home>Q<End>W')
  edit('edit-shift-left', '<S-Left>Z')
  edit('edit-shift-right', '<Home><S-Right>Z')
  edit('edit-del', '<Left><Left><Del>')
  edit('edit-ctrl-v-tab', '<C-v><Tab>"')
  edit('edit-ctrl-v-esc', '<C-v><Esc>"')
  edit('edit-ctrl-v-decimal', '<C-v>065"')
  edit('edit-ctrl-q', '<C-q><Tab>"')
  edit('edit-ctrl-k-digraph', '<C-k>Co"')
  edit('edit-ctrl-a', '<C-a>')
  edit('edit-ctrl-l', '<C-l>')
  edit('edit-ctrl-d', '<C-d>')
  edit('edit-ctrl-t', '<C-t>')
  edit('edit-ctrl-y', '<C-y>')
  edit('edit-ctrl-underscore', '<C-_>')
  edit('edit-ctrl-caret', '<C-^>')
  edit('edit-ctrl-r-ctrl-r', '<C-r><C-r>"', { 'call setreg("a", "RR")' })
  edit('edit-ctrl-r-ctrl-o', '<C-r><C-o>"', { 'call setreg("a", "RO")' })
  -- The wildmenu keys, on a line whose completion is deterministic.
  cmdcase('edit-wild-tab', {}, ':edit cdir/a<Tab><Esc>')
  cmdcase('edit-wild-tab-tab', {}, ':edit cdir/a<Tab><Tab><Esc>')
  cmdcase('edit-wild-shift-tab', {}, ':edit cdir/a<S-Tab><Esc>')
  cmdcase('edit-wild-ctrl-n', {}, ':edit cdir/a<Tab><C-n><Esc>')
  cmdcase('edit-wild-ctrl-p', {}, ':edit cdir/a<Tab><C-p><Esc>')
  cmdcase('edit-wild-ctrl-e', {}, ':edit cdir/a<Tab><C-e><Esc>')
  cmdcase('edit-wild-ctrl-y', {}, ':edit cdir/a<Tab><C-y><Esc>')
  cmdcase('edit-wild-ctrl-a', {}, ':edit cdir/a<C-a><Esc>')
  cmdcase('edit-wild-ctrl-l', {}, ':edit cdir/a<C-l><Esc>')
  cmdcase('edit-wild-ctrl-d', {}, ':edit cdir/a<C-d><Esc>')
  -- `<C-\>e` replaces the whole line with the value of an expression --
  -- the `command_line_execute` re-entry that has its own save/restore.
  cmdcase('edit-ctrl-bsl-e', {}, [[:abc<C-\>e'let g:q = 11'<CR><CR>]])
  cmdcase('edit-ctrl-bsl-e-err', {}, [[:abc<C-\>enosuchfn()<CR><CR><Esc>]])
  cmdcase('edit-ctrl-bsl-ctrl-n', {}, [[:abc<C-\><C-n>]])
  cmdunwatch()
end)

-- =====================================================================
-- 17. ex_getln.rs -- history, the command window and 'cedit'.
-- =====================================================================
section('cmdline-history', function()
  cmdwatch()
  for _, entry in ipairs({ 'let g:q = 1', 'let g:q = 2', 'echo "three"' }) do
    evalp('hist-add ' .. entry, string.format('histadd(":", %q)', entry))
  end
  evalp('hist-nr', 'histnr(":")')
  for _, index in ipairs({ -1, -2, -3, 0, 1, 99 }) do
    evalp('hist-get ' .. index, 'histget(":", ' .. index .. ')')
  end
  evalp('hist-get-name', 'histget(":", -1)')
  for _, name in ipairs({ ':', '/', '?', '=', '@', '>', 'cmd', 'search', 'expr', 'input', 'debug', 'nosuch' }) do
    evalp('hist-nr ' .. name, string.format('histnr(%q)', name))
  end
  evalp('hist-del-pat', 'histdel(":", "echo")')
  evalp('hist-after-del', 'histget(":", -1)')
  evalp('hist-del-all', 'histdel(":")')
  evalp('hist-empty', 'histget(":", -1)')
  evalp('hist-bad-name', 'histnr("nope")')
  evalp('hist-add-bad', 'histadd("nope", "x")')
  -- Recall keys.  Each is a `command_line_handle_key` arm over the
  -- history list; `<Up>` filters by the typed prefix, `<C-p>` does not.
  cmdcase('hist-recall-up', { 'call histadd(":", "let g:q = 21")' }, ':<Up><CR>')
  cmdcase('hist-recall-ctrl-p', { 'call histadd(":", "let g:q = 22")' }, ':<C-p><CR>')
  cmdcase('hist-recall-prefix', {
    'call histadd(":", "let g:q = 23")',
    'call histadd(":", "echo 1")',
  }, ':let<Up><CR>')
  cmdcase('hist-recall-down', {
    'call histadd(":", "let g:q = 24")',
    'call histadd(":", "let g:q = 25")',
  }, ':<Up><Up><Down><CR>')
  cmdcase('hist-recall-search', { 'call histadd("/", "second")' }, '/<Up><CR>')
  -- The command window.  `open_cmdwin` is 326 lines and reached three
  -- ways: `q:`, `q/` and 'cedit' from inside the line.
  cmdcase('cmdwin-colon', {}, 'q:ilet g:q = 31<CR>')
  cmdcase('cmdwin-search', {}, 'q/isecond<CR>')
  cmdcase('cmdwin-abort', {}, 'q:ilet g:q = 32<Esc><C-w>c')
  cmdcase('cmdwin-cedit', { 'set cedit=<C-f>' }, ':let g:q = 33<C-f><CR>')
  cmdcase('cmdwin-cedit-off', { 'set cedit=' }, ':let g:q = 34<C-f><CR>')
  cmdcase('cmdwin-from-search', { 'set cedit=<C-f>' }, '/second<C-f><CR>')
  evalp('cmdwin-type-outside', 'getcmdwintype()')
  cmdunwatch()
end)

-- =====================================================================
-- 18. ex_getln.rs -- setcmdline/setcmdpos and the prompt functions.
--
-- `input()`/`inputlist()`/`confirm()` are `get_user_input` +
-- `do_dialog`, and both halves matter: the argument-validation half is
-- pure error text, and the working half is driven with keys queued
-- *before* the call, which is where the inner vgetc looks.
-- =====================================================================
section('cmdline-api', function()
  cmdwatch()
  cmdcase('setcmdline-basic', {
    'autocmd CmdlineEnter * ++once call setcmdline("let g:q = 41")',
  }, ':<CR>')
  cmdcase('setcmdline-pos', {
    'autocmd CmdlineEnter * ++once call setcmdline("let g:q = 42", 5)',
  }, ':<CR>')
  cmdcase('setcmdpos-mid', {
    'autocmd CmdlineChanged * ++once call setcmdpos(2)',
  }, ':let g:q = 43<CR>')
  cmdunwatch()
  -- Argument validation.  Every one of these is an error text and
  -- nothing else, which makes it exactly the half a screen oracle
  -- cannot see.
  --
  -- `evalq` and not `evalp`: several of these argument lists are bad in
  -- a way `input()`/`confirm()` only notices *after* it has put the
  -- prompt up, and a prompt with an empty typeahead in script mode ends
  -- the run on the spot -- measured, `input(1)` truncated the section
  -- silently and exited 0 (docket D-B13-2).  Ten queued `<Esc>` cancel
  -- any prompt that does appear and are inert in normal mode; the
  -- leftovers are drained before the next case.
  local function evalq(label, expr)
    pcall(vim.fn.feedkeys, tc(('<Esc>'):rep(10)), 'n')
    evalp(label, expr)
    pcall(vim.fn.feedkeys, '', 'x')
    pcall(vim.cmd, 'silent! stopinsert')
  end
  for _, expr in ipairs({
    'setcmdline("x")',
    'setcmdline(1)',
    'setcmdline("x", -1)',
    'setcmdline("x", "y")',
    'setcmdpos(3)',
    'setcmdpos(-1)',
    'setcmdpos("x")',
    'getcmdline()',
    'getcmdpos()',
    'getcmdtype()',
    'getcmdscreenpos()',
    'getcmdcompltype()',
    'getcmdcomplpat()',
    'getcmdprompt()',
    'getcmdwintype()',
    'cmdcomplete_info()',
    'input()',
    'input(1)',
    'input({})',
    'input("a", "b", "nosuchcompl")',
    'input({"prompt": "p", "default": "d", "completion": "nosuch"})',
    'input({"prompt": 1})',
    'inputlist("x")',
    'inputlist([])',
    'inputlist([1, 2])',
    'inputsecret(1)',
    'confirm()',
    'confirm(1)',
    'confirm("m", "&a", "x")',
    'confirm("m", "&a", 1, "nosuchtype")',
    'histadd(1, "x")',
    'histget([])',
    'histdel(1)',
    'histnr([])',
    'inputsave()',
    'inputrestore()',
  }) do
    evalq('api ' .. expr, expr)
  end
  -- The working half, keys queued first.
  local function prompted(label, expr, pre)
    pcall(vim.fn.feedkeys, tc(pre), 't')
    local ok, res = pcall(vim.fn.eval, expr)
    answer(label, ok and res or ('ERROR ' .. scrub(tostring(res):gsub('.-:%s*Vim', 'Vim'))))
    feed('<Esc>')
  end
  prompted('input-typed', 'input("Say: ")', 'hello<CR>')
  prompted('input-default', 'input("Say: ", "pre")', '<CR>')
  prompted('input-cancel', 'input("Say: ")', 'abc<Esc>')
  prompted('input-completion', 'input("F: ", "cdir/a", "file")', '<Tab><CR>')
  prompted('input-dict', 'input({"prompt": "P: ", "default": "dd"})', '<CR>')
  prompted('inputsecret-typed', 'inputsecret("Pass: ")', 'sekrit<CR>')
  prompted('inputlist-pick', 'inputlist(["pick:", "1. a", "2. b"])', '2<CR>')
  prompted('inputlist-cancel', 'inputlist(["pick:", "1. a"])', '<CR>')
  prompted('confirm-pick', 'confirm("Really?", "&yes\n&no", 1)', 'y')
  prompted('confirm-default', 'confirm("Really?", "&yes\n&no", 2)', '<CR>')
  prompted('confirm-cancel', 'confirm("Really?", "&yes\n&no", 1)', '<Esc>')
  evalp('inputsave-restore', 'inputsave() . "," . inputrestore()')
end)

-- =====================================================================
-- 19. cmdexpand.rs -- the context walker.
--
-- `set_one_cmd_context` + `set_context_by_cmdname` decide which of the
-- 45 completion contexts a partial command line is in; `ExpandOther`
-- (435 lines) then answers it.  The candidate *lists* are recorded, not
-- just their sizes, because a walker that picks the wrong arm usually
-- keeps the count for most patterns -- phase-14's comp-probe learned
-- that the expensive way.  Long lists are digested (n + head + tail):
-- 'help' alone is several thousand entries and would swamp the report
-- without saying more than its ends do.
-- =====================================================================
local function complist(label, ...)
  local ok, res = pcall(vim.fn.getcompletion, ...)
  if not ok then
    local text = 'ERROR ' .. scrub(tostring(res):gsub('.-:%s*Vim', 'Vim'))
    emit(label, '=', esc(text))
    struct(label, text)
    return
  end
  -- Scrub BEFORE sorting, not after.  `canon` scrubs on the way into
  -- the `.struct` artifact, but the sort would already have ordered the
  -- list by the *unscrubbed* strings -- and `scriptnames` completion
  -- answers absolute paths, so the same binary produced two different
  -- orders from two different working directories.  It also puts the
  -- scrub on the `.txt` path, which `emit` does not do for a table.
  for index, item in ipairs(res) do
    res[index] = scrub(item)
  end
  table.sort(res)
  local digest = { n = #res }
  if #res <= 24 then
    digest.all = res
  else
    digest.head = vim.list_slice(res, 1, 8)
    digest.tail = vim.list_slice(res, #res - 3, #res)
  end
  struct(label, digest)
  emit(
    label,
    '= n=' .. #res,
    esc(vim.inspect(digest.all or digest.head):gsub('%s+', ' ')),
    digest.tail and ('.. ' .. esc(vim.inspect(digest.tail):gsub('%s+', ' '))) or ''
  )
end

section('complete-context', function()
  cmdfixture()
  exec('silent! let g:probe_a = 1')
  exec('silent! let g:probe_b = 2')
  exec('silent! command! -nargs=1 Probecmd echo <q-args>')
  exec('silent! command! -nargs=1 -complete=custom,ProbeCustom Probecust echo <q-args>')
  exec('silent! function! ProbeCustom(A, L, P)\nreturn "cx1\\ncx2\\ncx3"\nendfunction')
  exec('silent! command! -nargs=1 -complete=customlist,ProbeCustomL Probecustl echo <q-args>')
  exec('silent! function! ProbeCustomL(A, L, P)\nreturn ["cl1", "cl2"]\nendfunction')
  exec('silent! augroup ProbeGroup\nautocmd!\naugroup END')
  -- One line per arm of the walker.  Commands first, then the argument
  -- shapes each of them dispatches on.
  for _, line in ipairs({
    '', 's', 'se', 'set', 'set ', 'set no', 'set inv', 'set backspace',
    'set backspace=', 'set backspace+=', 'set backspace-=', 'set backspace^=',
    'setlocal ', 'setglobal ', 'set all&', 'set sh',
    'e', 'edit ', 'edit cdir/', 'edit cdir/a', 'edit ~/', 'edit $HOME/',
    'edit cdir/sub/', 'read ', 'write ', 'source ', 'runtime ',
    'b', 'buffer ', 'sbuffer ', 'bdelete ',
    'h', 'help ', 'help getcmd', 'helpgrep ',
    'hi', 'highlight ', 'highlight Nor', 'highlight link ',
    'sy', 'syntax ', 'syntax keyword ', 'syntax list ',
    'au', 'autocmd ', 'autocmd Buf', 'autocmd BufRead ', 'autocmd! ',
    'augroup ', 'doautocmd ', 'doautoall ',
    'map ', 'nmap ,', 'unmap ', 'mapclear ', 'abbrev ', 'cabbrev ',
    'command ', 'command! ', 'command -nargs=', 'command -complete=',
    'command -addr=', 'command -range=', 'delcommand ',
    'Probecmd ', 'Probecust ', 'Probecustl ',
    'let ', 'let g:', 'let g:probe_', 'unlet ', 'unlet g:probe_',
    'echo ', 'echo g:', 'echo g:probe_', 'echo &', 'echo &backs',
    'echo $', 'echo v:', 'echo getcm', 'call getcm', 'call ',
    'if ', 'while ', 'return ', 'throw ', 'for ',
    'lua ', 'lua vim.ap', 'lua =vim.', 'luado ', 'luafile ',
    'sign ', 'sign define ', 'sign place ',
    'menu ', 'amenu ', 'unmenu ',
    'history ', 'messages ', 'checkhealth ', 'packadd ',
    'colorscheme ', 'compiler ', 'filetype ', 'setfiletype ',
    'profile ', 'breakadd ', 'breakdel ', 'scriptnames ',
    'tag ', 'tselect ', 'ptag ',
    'cd ', 'lcd ', 'tcd ', 'cd cdir/',
    'normal ', 'silent ', 'verbose ', 'debug ', 'redir ',
    's/', 's/a/', 'g/', 'v/', 'g/a/', '1,2', '%s/a/b/',
    '!', '!l', 'r !', 'w !', 'terminal ',
    'argument ', 'args ', 'argadd ', 'argdelete ',
    'options', 'ownsyntax ', 'language ', 'behave ',
    'match ', 'syntime ', 'retab ', 'digraphs ',
    'wincmd ', 'tabmove ', 'mkview ', 'loadview ',
    'diffget ', 'diffput ', 'diffsplit ',
    'nohlsearch', 'ju', 'jumps', 'marks', 'registers ', 'display ',
    'set wildoptions=', 'set completeopt=', 'set shortmess=',
    'set messagesopt=', 'set wildmode=', 'set wildchar=',
  }) do
    complist('ctx ' .. (line == '' and '<empty>' or line), line, 'cmdline')
  end
  -- getcompletiontype() answers the same question without expanding.
  for _, line in ipairs({
    'set ', 'set backspace=', 'edit cdir/', 'help ', 'echo g:', 'call ',
    'nmap ', 'sign ', 'lua ', 's/a/', '!l', 'Probecust ',
  }) do
    evalp('ctxtype ' .. line, string.format('getcompletiontype(%q)', line))
  end
end)

-- =====================================================================
-- 20. cmdexpand.rs -- every named completion type.
--
-- `getcompletion(pat, type)` reaches `ExpandFromContext` directly, one
-- entry per row of the `command_complete` table.  Two patterns each:
-- the empty one takes the whole list, a prefix takes the filter.
-- =====================================================================
section('complete-types', function()
  cmdfixture()
  exec('silent! let g:probe_a = 1')
  exec('silent! function! ProbeCustomL(A, L, P)\nreturn ["cl1", "cl2"]\nendfunction')
  local TYPES = {
    'arglist', 'augroup', 'breakpoint', 'buffer', 'checkhealth', 'color',
    'command', 'compiler', 'diff_buffer', 'dir', 'dir_in_path', 'environment',
    'event', 'expression', 'file', 'file_in_path', 'filetype', 'filetypecmd',
    'function', 'help', 'highlight', 'history', 'keymap', 'locale', 'lua',
    'mapclear', 'mapping', 'menu', 'messages', 'option', 'packadd', 'retab',
    'runtime', 'scriptnames', 'shellcmd', 'shellcmdline', 'sign', 'syntax',
    'syntime', 'tag', 'tag_listfiles', 'user', 'var',
  }
  for _, kind in ipairs(TYPES) do
    complist('type ' .. kind .. ' <empty>', '', kind)
    complist('type ' .. kind .. ' a', 'a', kind)
  end
  -- The two types that need a function argument, plus the rejections.
  complist('type custom', '', 'custom,ProbeCustomL')
  complist('type customlist', '', 'customlist,ProbeCustomL')
  for _, bad in ipairs({
    'getcompletion("", "nosuchtype")',
    'getcompletion("", "custom")',
    'getcompletion("", "custom,NoSuchFn")',
    'getcompletion("", 1)',
    'getcompletion(1, "file")',
    'getcompletion("", "cmdline", "x")',
  }) do
    evalp('type-bad ' .. bad, bad)
  end
  -- The `filtered` third argument, and file completion under a cwd that
  -- is not the one the pattern names.
  evalp('type-filtered', 'len(getcompletion("cdir/", "file", v:true))')
  complist('type file cdir', 'cdir/', 'file')
  complist('type dir cdir', 'cdir/', 'dir')
  complist('type file_in_path a', 'a', 'file_in_path')
end)

-- =====================================================================
-- 21. cmdexpand.rs -- the wildmenu option matrix.
--
-- `nextwild` + `ExpandOne` read six options and `showmatches` reads two
-- more; none of them is varied anywhere else in the corpus.  Each case
-- is driven through the real key loop, so what is recorded is what the
-- *line* became -- the option's effect, not its value.
-- =====================================================================
section('complete-wild', function()
  cmdfixture()
  cmdwatch()
  for _, mode in ipairs({
    'full', 'longest', 'list', 'list:full', 'list:longest', 'longest:full',
    'list:lastused', 'full,full', 'longest,full', 'list,full', 'noselect',
    'longest:full,full', '', 'full:lastused',
  }) do
    cmdcase('wildmode ' .. (mode == '' and '<empty>' or mode), {
      'set wildmode=' .. mode,
    }, ':edit cdir/a<Tab><Esc>')
    cmdcase('wildmode-twice ' .. (mode == '' and '<empty>' or mode), {
      'set wildmode=' .. mode,
    }, ':edit cdir/a<Tab><Tab><Esc>')
  end
  for _, opts in ipairs({
    '', 'pum', 'tagfile', 'fuzzy', 'pum,tagfile', 'fuzzy,pum',
  }) do
    cmdcase('wildoptions ' .. (opts == '' and '<empty>' or opts), {
      'set wildoptions=' .. opts,
    }, ':edit cdir/a<Tab><Esc>')
  end
  cmdcase('wildoptions-fuzzy-cmd', { 'set wildoptions=fuzzy' }, ':stfl<Tab><Esc>')
  -- 'wildchar' and 'wildcharm': the first is the key that starts
  -- completion from the command line, the second the one that does it
  -- from *inside a mapping*, and they are separate code paths.
  cmdcase('wildchar-comma', { 'set wildchar=44' }, ':edit cdir/a,<Esc>')
  cmdcase('wildchar-esc', { 'set wildchar=27' }, ':edit cdir/a<Esc><Esc>')
  cmdcase('wildchar-tab-default', { 'set wildchar=9' }, ':edit cdir/a<Tab><Esc>')
  cmdcase('wildcharm-in-map', {
    'set wildcharm=<C-z>',
    'cnoremap ,w cdir/a<C-z>',
  }, ':edit ,w<Esc>')
  cmdcase('wildcharm-unset', {
    'set wildcharm=0',
    'cnoremap ,w cdir/a<C-z>',
  }, ':edit ,w<Esc>')
  -- 'wildignore' and 'wildignorecase' filter the answer, and the filter
  -- applies to the key loop and to getcompletion() differently.
  for _, ignore in ipairs({ '', '*.txt', '*.vim,*.log', 'cdir/*' }) do
    cmdcase('wildignore ' .. (ignore == '' and '<empty>' or ignore), {
      'set wildignore=' .. ignore,
    }, ':edit cdir/<Tab><Esc>')
    exec('silent! set wildignore=' .. ignore)
    complist('wildignore-fn ' .. (ignore == '' and '<empty>' or ignore), 'cdir/', 'file')
  end
  exec('silent! set wildignore=')
  for _, flag in ipairs({ 'wildignorecase', 'nowildignorecase' }) do
    exec('silent! set ' .. flag)
    complist('wildignorecase ' .. flag .. ' upper', 'cdir/A', 'file')
    complist('wildignorecase ' .. flag .. ' lower', 'cdir/a', 'file')
    cmdcase('wildignorecase ' .. flag, { 'set ' .. flag }, ':edit cdir/A<Tab><Esc>')
    -- 'longest' is the mode that runs find_longest_match, and the fold
    -- only shows up there: with the case-insensitive candidates in the
    -- fixture, `cdir/a` + 'wildignorecase' has a longer common prefix
    -- than `cdir/a` without it.
    cmdcase('wildignorecase-longest ' .. flag, {
      'set ' .. flag,
      'set wildmode=longest',
    }, ':edit cdir/a<Tab><Esc>')
    cmdcase('wildignorecase-longest-upper ' .. flag, {
      'set ' .. flag,
      'set wildmode=longest',
    }, ':edit cdir/A<Tab><Esc>')
  end
  exec('silent! set nowildignorecase')
  -- 'suffixes' and 'wildmenu' change the ordering and the display half.
  cmdcase('suffixes-vim', { 'set suffixes=.vim' }, ':edit cdir/a<C-a><Esc>')
  cmdcase('nowildmenu', { 'set nowildmenu' }, ':edit cdir/a<Tab><Esc>')
  cmdcase('wildmenu-pum', { 'set wildmenu wildoptions=pum' }, ':edit cdir/a<Tab><Esc>')
  cmdunwatch()
end)

-- =====================================================================
-- 22. insexpand.rs -- insert-mode completion.
--
-- Not a full insexpand oracle (that is B13-13/14's own problem), but
-- the half that is *answerable without a screen*: what the buffer holds
-- afterwards and what `complete_info()` says about the state that
-- produced it.
-- =====================================================================
local COMPL_LINES = {
  'alpha alphabet alphanumeric',
  'beta betamax',
  'gamma',
  'alp',
}

section('complete-ins', function()
  cmdfixture()
  -- Wipe first, every case.  `keycase` only does `enew!`, which *adds*
  -- a buffer -- and 'complete' scans the buffer list, so without this
  -- each case sees one more "Scanning: [No Name]" than the last and the
  -- whole section becomes a function of its own ordering (measured: the
  -- message column grew by one line per case).
  local function inscase(label, prep, keys)
    exec('silent! %bwipeout!')
    keycase(label, prep, keys, { lines = COMPL_LINES, cursor = { 4, 0 }, vars = { 'ci' } })
  end
  local INFO = '<C-r><C-r>=execute("let g:ci = complete_info()")<CR>'
  for _, opt in ipairs({
    'menu', 'menuone', 'menu,preview', 'menuone,noselect', 'menuone,noinsert',
    'longest', 'longest,menuone', 'popup', 'menu,popup', 'fuzzy,menuone',
    'noselect,noinsert', 'preinsert,menuone', '',
  }) do
    inscase('cpt-' .. (opt == '' and '<empty>' or opt), {
      'set completeopt=' .. opt,
    }, 'A<C-n><Esc>')
  end
  for _, keys in ipairs({
    { 'ins-ctrl-n', 'A<C-n><Esc>' },
    { 'ins-ctrl-p', 'A<C-p><Esc>' },
    { 'ins-ctrl-n-twice', 'A<C-n><C-n><Esc>' },
    { 'ins-ctrl-n-ctrl-e', 'A<C-n><C-e><Esc>' },
    { 'ins-ctrl-n-ctrl-y', 'A<C-n><C-y><Esc>' },
    { 'ins-ctrl-x-ctrl-n', 'A<C-x><C-n><Esc>' },
    { 'ins-ctrl-x-ctrl-p', 'A<C-x><C-p><Esc>' },
    { 'ins-ctrl-x-ctrl-l', 'A<C-x><C-l><Esc>' },
    { 'ins-ctrl-x-ctrl-f', 'ccdir/a<C-x><C-f><Esc>' },
    { 'ins-ctrl-x-ctrl-v', 'ccal getcm<C-x><C-v><Esc>' },
    { 'ins-ctrl-x-ctrl-d', 'A<C-x><C-d><Esc>' },
    { 'ins-ctrl-x-ctrl-i', 'A<C-x><C-i><Esc>' },
    { 'ins-ctrl-x-ctrl-t', 'A<C-x><C-t><Esc>' },
    { 'ins-ctrl-x-ctrl-k', 'A<C-x><C-k><Esc>' },
    { 'ins-ctrl-x-ctrl-s', 'A<C-x><C-s><Esc>' },
    { 'ins-ctrl-x-ctrl-o', 'A<C-x><C-o><Esc>' },
    { 'ins-ctrl-x-ctrl-u', 'A<C-x><C-u><Esc>' },
    { 'ins-ctrl-n-bs', 'A<C-n><BS><Esc>' },
    { 'ins-ctrl-n-space', 'A<C-n> x<Esc>' },
    { 'ins-ctrl-n-ctrl-n-ctrl-p', 'A<C-n><C-n><C-p><Esc>' },
  }) do
    inscase(keys[1], { 'set completeopt=menu,preview' }, keys[2])
  end
  -- 'complete' selects the sources; each flag is its own scanner.
  for _, cpt in ipairs({ '.', 'w', 'b', 'u', 'U', 'k', 's', 't', 'i', 'd', '.,w,b', '' }) do
    inscase('cpt-src-' .. (cpt == '' and '<empty>' or cpt), {
      'set completeopt=menu',
      'set complete=' .. cpt,
    }, 'A<C-n><Esc>')
  end
  -- completefunc / omnifunc, including the three ways a user function
  -- can answer (list, dict, -1/-2/-3 sentinels).
  exec([[
    silent! function! ProbeCompl(findstart, base)
      if a:findstart
        return 0
      endif
      return ['pf1', 'pf2', 'pf3']
    endfunction
  ]])
  exec([[
    silent! function! ProbeComplDict(findstart, base)
      if a:findstart
        return 0
      endif
      return {'words': [{'word': 'dw1', 'menu': 'M', 'info': 'I', 'kind': 'K'}], 'refresh': 'always'}
    endfunction
  ]])
  exec([[
    silent! function! ProbeComplBad(findstart, base)
      if a:findstart
        return -3
      endif
      return []
    endfunction
  ]])
  inscase('ins-completefunc', {
    'set completeopt=menu',
    'set completefunc=ProbeCompl',
  }, 'A<C-x><C-u><Esc>')
  inscase('ins-completefunc-dict', {
    'set completeopt=menu',
    'set completefunc=ProbeComplDict',
  }, 'A<C-x><C-u><Esc>')
  inscase('ins-completefunc-sentinel', {
    'set completeopt=menu',
    'set completefunc=ProbeComplBad',
  }, 'A<C-x><C-u><Esc>')
  inscase('ins-omnifunc', {
    'set completeopt=menu',
    'set omnifunc=ProbeCompl',
  }, 'A<C-x><C-o><Esc>')
  inscase('ins-omnifunc-missing', {
    'set completeopt=menu',
    'set omnifunc=NoSuchProbeFn',
  }, 'A<C-x><C-o><Esc>')
  -- complete() / complete_add() / complete_check(), called from an
  -- <expr> mapping so they run with `ctrl_x_mode` set the way the
  -- documentation requires.
  exec([[
    silent! function! ProbeStart()
      call complete(1, ['cA', 'cB', 'cC'])
      return ''
    endfunction
  ]])
  -- The mapping goes in `prep`, not here: `keycase` calls `clear_maps()`
  -- before it applies the prep list, so anything defined at section
  -- scope is gone by the time the keys are fed (measured -- the first
  -- draft typed ",c" into the buffer as literal text).
  local CMAP = { 'set completeopt=menu', 'inoremap <expr> ,c ProbeStart()' }
  inscase('ins-complete-fn', CMAP, 'A,c<Esc>')
  inscase('ins-complete-then-info', CMAP, 'A,c' .. INFO .. '<Esc>')
  for _, expr in ipairs({
    'complete(1, ["a"])',
    'complete(0, ["a"])',
    'complete("x", ["a"])',
    'complete(1, "a")',
    'complete_add("a")',
    'complete_add(1)',
    'complete_check()',
    'complete_info()',
    'complete_info(["mode"])',
    'complete_info(["mode", "pum_visible", "items", "selected", "inserted", "matches"])',
    'complete_info("x")',
    'complete_info([1])',
  }) do
    evalp('ins-api ' .. expr, expr)
  end
end)

-- =====================================================================
-- 23. message.rs -- the echo family.
--
-- `msg_puts_display`, `msg_outtrans*`, `emsg`/`msgmsg`/`smsg` and the
-- history.  In a headless process the text goes to stderr, so the
-- `.stderr` artifact is the only faithful view of the *rendering*;
-- `:redir` and `execute()` catch what the API can see, and the two do
-- not always agree, which is itself worth pinning.
-- =====================================================================
local function msgcase(label, cmd, opts)
  opts = opts or {}
  exec('silent! messages clear')
  vim.v.errmsg = ''
  vim.v.statusmsg = ''
  vim.v.warningmsg = ''
  -- Queue answers for a hit-enter prompt before the command runs.
  -- `execute()` sets msg_silent and *usually* suppresses the prompt, but
  -- not always -- `:intro` raised one and ended the section on the spot
  -- (script mode with an empty typeahead, docket D-B13-2).  `<CR>` is
  -- inert in normal mode, and the leftovers are drained below.
  pcall(vim.fn.feedkeys, tc(opts.pre or ('<CR>'):rep(6)), 'n')
  local captured = vcall('execute', cmd, opts.silent or '')
  pcall(vim.fn.feedkeys, '', 'x')
  local state = {
    cmd = cmd,
    captured = scrub(tostring(captured)),
    errmsg = scrub(vim.v.errmsg),
    statusmsg = scrub(vim.v.statusmsg),
    warningmsg = scrub(vim.v.warningmsg),
    history = scrub(vcall('execute', 'messages')),
  }
  struct(label, state)
  emit(
    label,
    '|',
    esc(cmd),
    '| cap=' .. esc(state.captured),
    '| err=' .. esc(state.errmsg),
    '| status=' .. esc(state.statusmsg),
    '| warn=' .. esc(state.warningmsg),
    '| hist=' .. esc(state.history)
  )
end

section('messages-echo', function()
  for _, case in ipairs({
    { 'echo-plain', 'echo "hello"' },
    { 'echo-empty', 'echo ""' },
    { 'echo-multi', 'echo "one" "two"' },
    { 'echo-newline', [[echo "a\nb"]] },
    { 'echo-tab', [[echo "a\tb"]] },
    { 'echo-ctrl', [[echo "a\<C-a>b"]] },
    { 'echo-nul', [[echo "a\x00b"]] },
    { 'echo-utf8', 'echo "α β ✓"' },
    { 'echo-latin1-byte', [[echo "a\xe9b"]] },
    { 'echo-long', 'echo repeat("x", 200)' },
    { 'echo-list', 'echo [1, 2, 3]' },
    { 'echo-dict', 'echo {"a": 1}' },
    { 'echo-float', 'echo 1.5' },
    { 'echo-blob', 'echo 0z00112233' },
    { 'echon-plain', 'echon "abc"' },
    { 'echon-after-echo', 'echo "one" | echon "two"' },
    { 'echomsg-plain', 'echomsg "kept"' },
    { 'echomsg-list', 'echomsg [1, 2]' },
    -- msg_hist_add strips leading and trailing newlines before the text
    -- enters the history, and nothing else in the corpus produces one:
    -- msg-hist-leading-newline was NOT CAUGHT without these two.
    { 'echomsg-leading-newline', [[echomsg "\nlead"]] },
    { 'echomsg-trailing-newline', [[echomsg "trail\n"]] },
    { 'echomsg-only-newlines', [[echomsg "\n\n"]] },
    { 'echoerr-plain', 'echoerr "bad"' },
    { 'echohl-then-echo', 'echohl WarningMsg | echo "warned" | echohl None' },
    { 'echohl-unknown', 'echohl NoSuchGroup | echo "x" | echohl None' },
    { 'echowindow', 'echowindow "win"' },
    { 'echoraw', 'echoraw "raw"' },
    { 'echo-expr-error', 'echo nosuchfn()' },
    { 'echomsg-expr-error', 'echomsg nosuchvar' },
    { 'unlet-missing', 'unlet g:nosuchvarhere' },
    { 'redir-var', 'redir => g:redir | echo "captured" | redir END' },
    { 'redir-append', 'redir => g:redir | echo "a" | redir END | redir =>> g:redir | echo "b" | redir END' },
    { 'redir-reg', 'redir @z | echo "toreg" | redir END' },
    { 'redir-nested-error', 'redir => g:redir | redir => g:redir2 | redir END' },
    { 'silent-echo', 'silent echo "quiet"' },
    { 'silent-echomsg', 'silent echomsg "quiethist"' },
    { 'silent-bang-error', 'silent! nosuchcommand' },
    { 'silent-error', 'silent nosuchcommand' },
    { 'verbose-echo', 'verbose echo "v"' },
    { 'messages-clear', 'messages clear' },
    { 'messages-after-clear', 'messages' },
  }) do
    msgcase(case[1], case[2])
  end
  evalp('redir-value', 'get(g:, "redir", "<unset>")')
  evalp('redir-reg-value', 'getreg("z")')
  -- The three v: message variables, set and read back.
  for _, name in ipairs({ 'errmsg', 'statusmsg', 'warningmsg' }) do
    exec(string.format('silent! let v:%s = "set-%s"', name, name))
    evalp('vmsg ' .. name, 'v:' .. name)
  end
  -- `execute()` with each silent flag, over a command that writes to
  -- both the message area and the history.
  for _, flag in ipairs({ '', 'silent', 'silent!', 'nosilent' }) do
    evalp('execute-flag ' .. (flag == '' and '<empty>' or flag),
      string.format('execute(%q, %q)', 'echomsg "flagged"', flag))
  end
  evalp('execute-list', 'execute(["echo 1", "echo 2"])')
  evalp('execute-bad', 'execute(1)')
  evalp('execute-nested', 'execute("execute(\'echo 1\')")')
end)

-- =====================================================================
-- 24. message.rs -- 'shortmess' and 'messagesopt'.
--
-- Each 'shortmess' flag suppresses one specific message, and the only
-- way to prove a flag is still wired is to produce that message with
-- the flag on and with it off.  `msg_history_len`/`msg_hit_return` come
-- from 'messagesopt', which nothing else in the corpus varies.
-- =====================================================================
section('messages-shortmess', function()
  cmdfixture()
  local FLAG_CASES = {
    { 'f', 'silent! edit ' .. work .. '/one.txt | argadd ' .. work .. '/two.txt | argument 1' },
    { 'i', 'silent! edit ' .. work .. '/one.txt' },
    { 'l', 'silent! edit ' .. work .. '/one.txt' },
    { 'm', 'silent! edit ' .. work .. '/one.txt | set modified' },
    { 'n', 'silent! new | set buftype=nofile' },
    { 'r', 'silent! edit ' .. work .. '/one.txt | set readonly' },
    { 'w', 'silent! edit! ' .. work .. '/one.txt | write' },
    { 'x', 'silent! edit ' .. work .. '/one.txt' },
    { 'a', 'silent! edit ' .. work .. '/one.txt' },
    { 'o', 'silent! edit! ' .. work .. '/one.txt' },
    { 'O', 'silent! edit! ' .. work .. '/one.txt' },
    { 's', 'silent! normal! /nosuchpattern' },
    -- 4,000 and not 300: `room` is
    -- `(Rows - cmdline_row - 1) * Columns + sc_col - 1`, about 1,800 on
    -- the default headless 80x24 grid, and `msg_may_trunc` does nothing
    -- to a message shorter than that.  At 300 the flag was wired to
    -- nothing observable (msg-trunc-shortmess, NOT CAUGHT).
    { 't', 'echo repeat("y", 4000)' },
    { 'T', 'echomsg repeat("z", 4000)' },
    { 'W', 'silent! edit! ' .. work .. '/one.txt | write' },
    { 'A', 'silent! edit ' .. work .. '/one.txt' },
    { 'I', 'intro' },
    { 'c', 'silent! normal! ggVGy' },
    { 'C', 'echo "c"' },
    { 'q', 'silent! normal! v' },
    { 'F', 'silent! edit ' .. work .. '/two.txt' },
    { 'S', 'silent! normal! /alpha' },
  }
  for _, case in ipairs(FLAG_CASES) do
    for _, shortmess in ipairs({ '', case[1] }) do
      exec('silent! %bwipeout!')
      exec('silent! enew!')
      exec('silent! set shortmess=' .. shortmess)
      msgcase(
        'shortmess ' .. case[1] .. ' ' .. (shortmess == '' and 'off' or 'on'),
        case[2]
      )
    end
  end
  exec('silent! set shortmess=filnxtToOF')
  exec('silent! %bwipeout!')
  exec('silent! enew!')
  -- 'messagesopt': the history cap and the hit-enter flag.
  for _, value in ipairs({
    'hit-enter,history:500', 'hit-enter,history:0', 'hit-enter,history:2',
    'history:500', 'wait:0,history:500', 'wait:100,history:500',
    'hit-enter,history:500,progress:c', 'hit-enter,history:500,progress:',
  }) do
    local ok = pcall(function()
      vim.o.messagesopt = value
    end)
    emit('messagesopt set ' .. value, '=', tostring(ok))
    if ok then
      exec('silent! messages clear')
      for index = 1, 6 do
        exec('silent! echomsg "m' .. index .. '"')
      end
      answer('messagesopt hist ' .. value, scrub(vcall('execute', 'messages')))
    end
  end
  for _, bad in ipairs({ 'nosuch', 'history:-1', 'history:x', 'wait:-1', 'progress:z' }) do
    evalp('messagesopt bad ' .. bad, string.format('execute("set messagesopt=%s")', bad))
  end
  pcall(function()
    vim.o.messagesopt = 'hit-enter,history:500'
  end)
end)

-- =====================================================================
-- 25. message.rs -- the error texts.
--
-- ~70 deliberately bad commands.  For most of them the error message is
-- the *only* observable half of the rejection: nothing changes, nothing
-- is printed, and a rewrite that raises E475 where the C raised E474 is
-- byte-identical under every other gate in the batch.
-- =====================================================================
section('messages-errors', function()
  for _, cmd in ipairs({
    'nosuchcommand',
    'nosuchcommand arg',
    'NoSuchCommand',
    'echo',
    'echo nosuchvar',
    'echo nosuchfn()',
    'echo 1 +',
    'echo "unterminated',
    "echo 'unterminated",
    'echo [1, 2',
    'echo {"a":',
    'let',
    'let g:',
    'let 1 = 2',
    'let g:x .= {}',
    'unlet g:nosuch',
    'unlet! g:nosuch',
    'call',
    'call nosuchfn()',
    'call strlen()',
    'call strlen("a", "b")',
    'if',
    'endif',
    'else',
    'elseif 1',
    'while',
    'endwhile',
    'for',
    'endfor',
    'try',
    'endtry',
    'catch',
    'finally',
    'return',
    'throw',
    'function',
    'endfunction',
    'delfunction NoSuchFn',
    'normal',
    'set nosuchoption',
    'set nosuchoption=1',
    'set backspace=nosuchvalue',
    'set backspace+=nosuchvalue',
    'set timeoutlen=abc',
    'set timeoutlen=-1',
    'set shortmess=Q',
    'set wildmode=nosuch',
    'set completeopt=nosuch',
    'set wildoptions=nosuch',
    'set cedit=nosuchkey',
    'edit',
    'edit /nosuch/dir/file',
    'buffer 9999',
    'bdelete 9999',
    'normal! 9999G',
    'help nosuchhelptagxyz',
    'tag nosuchtagxyz',
    'syntax nosuchsubcommand',
    'highlight NoSuchGroup nosuchkey=1',
    'highlight link',
    'autocmd NoSuchEvent * echo 1',
    'augroup',
    'map',
    'nmap <nosuchmod>x y',
    'unmap ,nosuchmapping',
    'abclear!',
    'sign define',
    'sign nosuchsub',
    'menu',
    'command',
    'command! -nargs=nosuch Foo echo 1',
    'command! -complete=nosuch Foo echo 1',
    'delcommand NoSuchCmd',
    'history nosuch',
    'messages nosuch',
    's/nosuchpatternxyz/x/',
    's/\\(/x/',
    'g/\\%(/d',
    'sort nosuchflag',
    '1,2,3print',
    '99999print',
    'wincmd nosuch',
    'lua nosuch_lua_global.x = 1',
    'luafile /nosuch/file.lua',
    'source /nosuch/file.vim',
    'runtime nosuch/file.vim',
    'redir END',
    'redir => 1',
    'q!!',
    'X',
    '&&&',
    '@',
  }) do
    run('errtext ' .. cmd, cmd)
    struct('errtext ' .. cmd, { cmd = cmd, out = exec(cmd), errmsg = scrub(vim.v.errmsg) })
    vim.v.errmsg = ''
  end
end)

-- =====================================================================
-- 26. message.rs -- the `--More--` pager, driven against the child.
--
-- This section is in-process nowhere, and the reason is a guard, not a
-- deadlock.  `do_more_prompt` opens with
--
--     no_need_more = headless_mode && !embedded_mode && !ui_active()
--
-- so in a plain `--headless -l` process it returns before it starts and
-- the pager is *unreachable* -- the first draft of this section drove it
-- with pre-queued keys in-process, every case answered differently, and
-- all of the difference was the leftover keys being executed in normal
-- mode afterwards.  A `--embed --headless` child clears the same guard
-- (embedded_mode is set), so the pager runs there with no UI attached,
-- and `nvim_get_mode()` reports `mode = "rm"`, `blocking = true` while
-- it holds the loop -- which is the same `fast`-request-only discipline
-- sections 11-12 already use.
--
-- `wait_return` is NOT reachable this way: its guard is
-- `headless_mode && !ui_active()`, which an embedded child without an
-- attached UI still satisfies.  The hit-enter prompt therefore belongs
-- to a probe that attaches a UI -- i.e. to scrsweep's ext_messages
-- scenarios, not here.  Measured both ways before it was written down.
-- =====================================================================
section('messages-pager', function()
  if not chan and not child_start() then
    emit('messages-pager !! child nvim would not start')
    return
  end
  cwait(200)
  local LONG = 'echo join(map(range(1, 90), {_, i -> printf("line %03d", i)}), "\n")'
  -- Build the history with 'nomore' in force, so filling it does not
  -- itself raise the prompt the case is about.  It has to be a real
  -- `:echomsg`: `execute()` and `:silent` both keep it out of the
  -- history in this tree (section 23 records that).
  local FILL = 'set nomore | for i in range(1, 90) | echomsg "hist ".i | endfor'
  --- The command is a *notification*: it does not return until the
  --- pager lets go, and an rpcrequest for it deadlocks the parent.
  local function pcase(label, prep, cmd, inputs, fill)
    cprep(prep)
    -- The fill runs here and not in `prep`, because cprep prefixes every
    -- prep line with `silent!` -- and `:silent echomsg` does not reach
    -- the message history at all, so a "fill" applied that way left
    -- `:messages` empty and the case never paged.
    if fill then
      creq('nvim_command', fill)
    end
    cnotify('nvim_command', cmd)
    cwait(250)
    local trace = {}
    local blocked, mode = cblocking()
    trace[#trace + 1] = ('start blocking=%s mode=%s'):format(tostring(blocked), tostring(mode))
    for _, keys in ipairs(inputs) do
      -- Stop feeding once the pager has let go.  A trailing `q` sent to
      -- a child that is back in normal mode starts a *recording* and
      -- parks it waiting for a register name, which csnap then reports
      -- as WEDGED -- five cases read that way before this guard, and
      -- none of it was about the pager.
      if blocked == false then
        trace[#trace + 1] = ('skip %q (already released)'):format(keys)
      else
        creq('nvim_input', keys)
        cwait(180)
        local after_mode
        blocked, after_mode = cblocking()
        trace[#trace + 1] = ('after %q blocking=%s mode=%s'):format(
          keys,
          tostring(blocked),
          tostring(after_mode)
        )
      end
    end
    csnap(label, { trace = trace, pager_cmd = cmd })
    for _, line in ipairs(trace) do
      emit(label, '  ~', esc(line))
    end
  end
  local MORE = { 'set more' }
  pcase('more-quit', MORE, LONG, { 'q' })
  pcase('more-space', MORE, LONG, { ' ', ' ', 'q' })
  pcase('more-return', MORE, LONG, { '<CR>', '<CR>', 'q' })
  pcase('more-down', MORE, LONG, { 'j', 'j', 'q' })
  pcase('more-up', MORE, LONG, { 'j', 'k', 'q' })
  pcase('more-forward-page', MORE, LONG, { 'f', 'f', 'q' })
  pcase('more-back-page', MORE, LONG, { 'f', 'b', 'q' })
  pcase('more-half-down', MORE, LONG, { 'd', 'd', 'q' })
  pcase('more-half-up', MORE, LONG, { 'd', 'u', 'q' })
  pcase('more-to-end', MORE, LONG, { 'G', 'q' })
  pcase('more-to-start', MORE, LONG, { 'G', 'g', 'q' })
  pcase('more-esc', MORE, LONG, { '<Esc>' })
  pcase('more-ctrl-c', MORE, LONG, { '<C-c>' })
  pcase('more-ctrl-y', MORE, LONG, { '<C-y>', 'q' })
  pcase('more-colon', MORE, LONG, { ':', 'let g:cmd = 88<CR>', 'q' })
  pcase('more-unknown-key', MORE, LONG, { 'Z', 'q' })
  pcase('more-off', { 'set nomore' }, LONG, { 'q' })
  pcase('more-short-output', MORE, 'echo "one\ntwo"', { 'q' })
  pcase('more-messages', MORE, 'messages', { ' ', ' ', 'q' }, FILL)
  pcase('more-messages-off', { 'set nomore' }, 'messages', { 'q' }, FILL)
  -- A long *single* line rather than many lines: the wrap path into the
  -- same prompt, and the one `msg_puts_display` scrolls rather than
  -- line-feeds into.
  pcase('more-wrapped-line', MORE, 'echo repeat("W ", 1200)', { ' ', 'q' })
  pcase('more-error-flood', MORE,
    'for i in range(1, 40) | echomsg "e".i | endfor', { ' ', 'q' })
end)

if chan then
  pcall(vim.fn.jobstop, chan)
  cwait(100)
end
if #child_stderr > 0 then
  io.stderr:write(table.concat(child_stderr, '\n'), '\n')
end

emit('')
emit('===== end =====')
emit('sections run:', sections_run)
structfd:close()
