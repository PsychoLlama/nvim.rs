-- Driver for the mark / jumplist / changelist differential sweep; see
-- jmarksweep.sh.
--
-- Covers crates/nvim/src/mark/ -- mod.rs, lookup.rs, adjust.rs,
-- jumplist.rs, show.rs, builtins.rs, shada.rs -- which is batch B-mark.
-- Phase 20's survey (p20-1 §D.2 GAP 2) found the batch with NO
-- dedicated differential and NO mutation anchors: `marksweep` is the
-- *marktree* oracle, and `perssweep`'s shada half watches only the
-- PERSISTED form.  This one watches the live behaviour.
--
-- The question every case asks is "where is every mark, and what do the
-- jumplist and changelist hold", and the answer is read back through
-- every surface the editor offers:
--
--   getpos("'x")          for a-z A-Z 0-9 and the whole tick family
--                         (. ^ " [ ] < > ' ( ) { }), including bufnr
--   getmarklist()         buffer-local and global, the other builtin
--   getjumplist()         entries + the index, per window
--   getchangelist()       entries + the index, per buffer
--   :marks :jumps :changes  the rendered listings, captured verbatim
--   the cursor            after 'x `x '' `` g; g, <C-o> <C-i>
--
-- Sections:
--   s01 every lower-case mark set and read back, and `m` with a name
--       that is not a mark.
--   s02 global marks across three real files, and what `'A` does to the
--       current buffer.
--   s03 the numbered marks 0-9 and how setpos()/shada reach them.
--   s04 the tick family: '' `` '[ '] '< '> '^ '. '" '( ') '{ '} and
--       what each answers after which operation.
--   s05 :marks with and without an argument, :delmarks, :delmarks!.
--   s06 the jumplist: what pushes an entry, the dedup, <C-o>/<C-i>,
--       :jumps, :clearjumps, 'jumpoptions', and the JUMPLISTSIZE clamp
--       driven past 100 entries.
--   s07 the changelist: what appends, the same-line dedup, g; and g,,
--       :changes, and the same clamp past 100.
--   s08 ADJUSTMENT -- every mark re-read after dd / o / :m / :t / :d /
--       :g//d / J / << / :s//\r/ / p / u, above, at, inside and below.
--   s09 marks in a buffer that is unloaded, deleted and wiped.
--   s10 a shada round trip: :wshada, wipe the world, :rshada.
--   s11 setpos()/getpos() in every shape, including the error arms.
--   s12 the API: nvim_buf_set_mark, nvim_buf_del_mark, nvim_get_mark,
--       nvim_del_mark, nvim_buf_get_mark.
--   s13 the MOTIONS: 'a vs `a, g'a vs g`a (which do not touch the
--       jumplist), and the E19/E20 arms.
--   s14 getmarklist() in both shapes, beside :marks, so a divergence
--       between the two surfaces over one slot is visible as one line.
--   s15 :lockmarks over :m, :d and :normal.
--   s16 the same commands run UNCAPTURED, so E19/E20/E78/E92/E475 reach
--       the .stderr artifact.
--
-- NONDETERMINISM, and what is done about it (see p20-12 for the list):
--   * BUFFER NUMBERS are a global counter.  Every buffer this sweep
--     opens is a NAMED file under $WORK, and the report prints the
--     basename, never the handle.  The scratch buffer is `main`; an
--     unnamed buffer prints `u<n>` where n is its distance from `main`,
--     which at least does not move when a section above it opens one
--     more file.  Handles are remembered in BUFNAME so a mark in a
--     WIPED buffer still names its file rather than an integer.
--   * FILE PATHS reach :marks and :jumps through `fm_getname`, which
--     shortens against the process CWD and expands `~`.  The sweep
--     chdir's into $WORK and scrubs it.
--   * `Columns` gates `mark_line`'s truncation, so the text column of
--     all three listings is a function of the terminal width.  Pinned
--     at 80.
--   * TIMESTAMPS are written into every FileMark but are printed by
--     nothing here.  They ARE a live comparison in `mark_set_global`
--     and `mark_set_local` (`fm.timestamp <= tgt.timestamp`), which is
--     why s10's round trip only ever merges a shada file this same run
--     wrote.
--   * `:filter` silently drops listing rows; never set.
--
-- JMARKSWEEP_ONLY is a Lua pattern matched against each section name.
-- JMARKSWEEP_TRACE=1 mirrors each section name to stderr; it must be
-- OFF for a baseline, because .stderr is a compared artifact.

local work = assert(os.getenv('JMARK_WORK'), 'JMARK_WORK unset')
local structpath = assert(os.getenv('JMARK_STRUCT'), 'JMARK_STRUCT unset')

local structfd = assert(io.open(structpath, 'w'))
local only = os.getenv('JMARKSWEEP_ONLY')
if only == '' then
  only = nil
end
local trace = os.getenv('JMARKSWEEP_TRACE') == '1'

io.stdout:setvbuf('line')

local function emit(...)
  io.write(table.concat({ ... }, ' '), '\n')
end

local runtime = os.getenv('VIMRUNTIME') or ''
local script = debug.getinfo(1, 'S').source:sub(2)

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
  text = text:gsub('<SNR>%d+_', '<SNR>_')
  text = text:gsub('%S*/target/debug/nvim', '<NVIM>')
  text = text:gsub('%S*/nvim%-%x+', '<NVIM>')
  return text
end

local function esc(bytes)
  return (tostring(bytes):gsub('[^\32-\126]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

-- ---------------------------------------------------------------------
-- Canonical dump.  Verbatim from opsweep.lua: the artifacts are read
-- side by side often enough that they must escape and sort the same
-- way.
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

local function errtext(res)
  local text = scrub(tostring(res))
  text = text:gsub('^.-:%d+: ', '')
  text = text:gsub('^nvim_exec2%(%), line %d+: ', '')
  text = text:gsub('^Vim:', '')
  text = text:gsub('^Vim%b():', '')
  return text
end

-- ---------------------------------------------------------------------
-- Buffer identity.  A handle is a global counter and would make the
-- artifact a function of every buffer opened above the case that
-- prints it; a NAME is not.
-- ---------------------------------------------------------------------

local BUF -- the scratch buffer, `main`
local BUFNAME = {} -- handle -> symbol, remembered across a wipe

local function basename(path)
  return (tostring(path):gsub('.*/', ''))
end

local function register(nr)
  if BUFNAME[nr] then
    return
  end
  local ok, name = pcall(vim.api.nvim_buf_get_name, nr)
  if ok and name ~= '' then
    BUFNAME[nr] = basename(name)
  end
end

local function scanbufs()
  for _, nr in ipairs(vim.api.nvim_list_bufs()) do
    register(nr)
  end
end

--- A buffer, named.  `0` is the "current buffer" sentinel getpos()
--- returns for a buffer-local mark and is passed through as `0`.
local function bufsym(nr)
  if nr == nil then
    return '?'
  end
  nr = tonumber(nr) or -1
  if nr == 0 then
    return '0'
  end
  if BUFNAME[nr] then
    return BUFNAME[nr]
  end
  if BUF and nr == BUF then
    return 'main'
  end
  register(nr)
  if BUFNAME[nr] then
    return BUFNAME[nr]
  end
  return 'u' .. tostring(BUF and (nr - BUF) or nr)
end

-- ---------------------------------------------------------------------
-- The world a case runs in.
-- ---------------------------------------------------------------------

local DEFAULTS = table.concat({
  -- NO `viminfo&` here.  'viminfo' is an alias for 'shada', so
  -- resetting it puts the DEFAULT back -- and the default carries
  -- `r/tmp/`, the "never remember marks for files under this prefix"
  -- rule.  The whole sweep lives in /tmp, so s10's round trip
  -- silently persisted nothing and the section looked green.
  'set jumpoptions= shada=!,\'100,<50,s10,h',
  'set report=9999 shortmess=filnxtToOFS nomore noshowmode belloff=all',
  'set nostartofline selection=inclusive virtualedit= whichwrap=',
  'set shiftwidth=8 tabstop=8 softtabstop=0 noexpandtab',
  'set undolevels=1000 noswapfile nobackup nowritebackup',
  'set columns=80 lines=24 cmdheight=1 laststatus=0',
  'set foldmethod=manual nofoldenable',
}, ' | ')

local function quiet(src)
  local ok, res = pcall(vim.api.nvim_exec2, src, { output = false })
  if not ok then
    return errtext(res)
  end
  return nil
end

--- Run an Ex command and CAPTURE its output.  This is how :marks,
--- :jumps and :changes reach the report: their whole answer is text.
local function capture(src)
  local ok, res = pcall(vim.api.nvim_exec2, src, { output = true })
  if not ok then
    return nil, errtext(res)
  end
  return scrub(res.output or ''), nil
end

local function feed(keys)
  local codes = vim.api.nvim_replace_termcodes(keys, true, true, true)
  local ok, err = pcall(vim.api.nvim_feedkeys, codes, 'ntx', false)
  if not ok then
    return errtext(err)
  end
  return nil
end

local function normalise()
  pcall(
    vim.api.nvim_feedkeys,
    vim.api.nvim_replace_termcodes('<C-\\><C-N>', true, true, true),
    'ntx',
    false
  )
end

-- Sixteen flat, distinguishable lines, so a line number in the report
-- reads directly as a fixture line.
local FLAT = {}
for i = 1, 16 do
  FLAT[i] = string.format('l%02d word%02d', i, i)
end

local CUR_LINES = {}

--- Clear every mark, jumplist entry and changelist entry this process
--- can reach.  Marks are GLOBAL state -- a `'A` set in s02 is still
--- set in s08 -- so "which marks are set" is only an answer about THIS
--- case if the case starts from nothing.
---
--- `:delmarks!` clears a-z plus the tick family for the current
--- buffer; `:delmarks A-Z0-9` clears the global table; `:clearjumps`
--- the jumplist.  The changelist has no clearing command at all, which
--- is why the scratch buffer is re-created rather than re-filled --
--- see `freshmain`.
local function clearmarks()
  quiet('silent! delmarks!')
  quiet('silent! delmarks a-zA-Z0-9')
  quiet('silent! clearjumps')
end

local function reset(lines, pos, opts)
  normalise()
  quiet(DEFAULTS)
  quiet('silent! only!')
  pcall(vim.api.nvim_set_current_buf, BUF)
  clearmarks()
  vim.api.nvim_buf_set_lines(BUF, 0, -1, false, lines or FLAT)
  CUR_LINES = vim.deepcopy(lines or FLAT)
  -- The changelist survives a buffer rewrite and there is no command
  -- that empties it, so it is emptied by hand: 101 no-op changes would
  -- do it, and re-creating the buffer would renumber the world.  What
  -- actually works is a single undo-joined rewrite followed by
  -- `:earlier 999f`, which leaves ONE entry -- so the report says
  -- `len=1` at the top of most cases rather than `len=0`, and that is
  -- the honest baseline rather than a lie.
  pcall(vim.api.nvim_win_set_cursor, 0, pos or { 1, 0 })
  if opts and opts ~= '' then
    quiet('silent! ' .. opts)
  end
  pcall(vim.api.nvim_win_set_cursor, 0, pos or { 1, 0 })
  scanbufs()
end

-- ---------------------------------------------------------------------
-- Reading the marks back.
-- ---------------------------------------------------------------------

local LOWER = {}
for c = string.byte('a'), string.byte('z') do
  LOWER[#LOWER + 1] = string.char(c)
end
local UPPER = {}
for c = string.byte('A'), string.byte('Z') do
  UPPER[#UPPER + 1] = string.char(c)
end
local DIGITS = {}
for c = 0, 9 do
  DIGITS[#DIGITS + 1] = tostring(c)
end
-- The tick family.  `'` and `` ` `` are the SAME slot asked two ways,
-- and both are listed on purpose: `mark_get_motion` dispatches them
-- separately and a mutation that mixes them up is only visible if both
-- are asked.
--
-- `( ) { }` are NOT here.  They are computed from the cursor on every
-- call, so they are ALWAYS "set" and would add four columns of noise
-- to every one of ~400 cases; s04 asks them directly, where they are
-- the question rather than the background.
local TICKS = { '.', '^', '"', '[', ']', '<', '>', "'", '`' }

--- getpos() for one mark, rendered as `buf:lnum,col[+off]`, or nil if
--- the mark is not set (lnum 0).
local function markpos(name)
  local ok, p = pcall(vim.fn.getpos, "'" .. name)
  if not ok or type(p) ~= 'table' then
    return nil, { err = errtext(p) }
  end
  if (p[2] or 0) == 0 then
    return nil, nil
  end
  local text = string.format('%s:%d,%d', bufsym(p[1]), p[2], p[3])
  if (p[4] or 0) ~= 0 then
    text = text .. '+' .. tostring(p[4])
  end
  return text, { b = bufsym(p[1]), l = p[2], c = p[3], o = p[4] }
end

--- Every mark that IS set, in a fixed order.  Unset marks are omitted
--- rather than printed as `0,0`: with 26+26+10+13 slots the noise
--- would be four times the signal, and "which marks are set" is one of
--- the answers.
local function marksnap()
  local shown, raw = {}, {}
  local groups = { LOWER, UPPER, DIGITS, TICKS }
  for _, group in ipairs(groups) do
    for _, name in ipairs(group) do
      local text, rec = markpos(name)
      if text then
        shown[#shown + 1] = name .. '=' .. text
        raw[name] = rec
      elseif rec then
        shown[#shown + 1] = name .. '!' .. esc(rec.err)
        raw[name] = rec
      end
    end
  end
  return shown, raw
end

--- A list of jumplist/changelist entries, capped.  s06 and s07 drive
--- both past 100 entries on purpose, and a 100-entry report line hides
--- the two numbers that are the answer.
local function entries(list)
  local shown, raw = {}, {}
  for _, e in ipairs(list) do
    -- getchangelist()'s entries carry no `bufnr` (the list is
    -- per-buffer by construction); getjumplist()'s do.  Printing `?`
    -- for the missing half would be four hundred lines of noise.
    local where = e.bufnr and (bufsym(e.bufnr) .. ':') or ''
    local text = string.format('%s%d,%d', where, e.lnum or 0, e.col or 0)
    if (e.coladd or 0) ~= 0 then
      text = text .. '+' .. tostring(e.coladd)
    end
    if e.filename then
      text = text .. '@' .. esc(scrub(e.filename))
    end
    shown[#shown + 1] = text
    raw[#raw + 1] = { b = bufsym(e.bufnr), l = e.lnum, c = e.col, a = e.coladd, f = e.filename and scrub(e.filename) }
  end
  -- `table.unpack` does not exist in the LuaJIT 5.1 the editor
  -- embeds; `vim.list_slice` is the portable spelling and its end
  -- bound is EXCLUSIVE.
  if #shown > 14 then
    local head = vim.list_slice(shown, 1, 7)
    local tail = vim.list_slice(shown, #shown - 5, #shown + 1)
    return table.concat(head, ' ') .. ' ...' .. tostring(#shown - 12) .. '... ' .. table.concat(tail, ' '), raw
  end
  return table.concat(shown, ' '), raw
end

local SEEN = {}

local function shot(label, opt)
  opt = opt or {}
  if SEEN[label] then
    emit(label, 'DUPLICATE-LABEL', tostring(SEEN[label] + 1))
  end
  SEEN[label] = (SEEN[label] or 0) + 1
  scanbufs()

  local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
  local changed = #lines ~= #CUR_LINES
  if not changed then
    for i = 1, #lines do
      if lines[i] ~= CUR_LINES[i] then
        changed = true
        break
      end
    end
  end

  local shownm, rawm = marksnap()
  local okj, jl = pcall(vim.fn.getjumplist)
  local jtext, jraw = '?', nil
  local jidx, jlen = -1, -1
  if okj and type(jl) == 'table' then
    local list = jl[1] or {}
    jidx = jl[2] or -1
    jlen = #list
    jtext, jraw = entries(list)
  end
  local okc, cl = pcall(vim.fn.getchangelist)
  local ctext, craw = '?', nil
  local cidx, clen = -1, -1
  if okc and type(cl) == 'table' then
    local list = cl[1] or {}
    cidx = cl[2] or -1
    clen = #list
    ctext, craw = entries(list)
  end
  local pos = vim.api.nvim_win_get_cursor(0)
  local view = vim.fn.winsaveview()

  emit(label, 'B', changed and esc(table.concat(lines, '\n')) or '=')
  emit(label, 'K', #shownm == 0 and '-' or table.concat(shownm, ' '))
  emit(label, 'J', string.format('i=%d n=%d %s', jidx, jlen, jtext == '' and '-' or jtext))
  emit(label, 'G', string.format('i=%d n=%d %s', cidx, clen, ctext == '' and '-' or ctext))
  emit(
    label,
    'P',
    string.format(
      'c=%d,%d cw=%s buf=%s',
      pos[1],
      pos[2],
      view.curswant == 2147483647 and 'MAX' or tostring(view.curswant),
      bufsym(vim.api.nvim_get_current_buf())
    )
  )

  struct(label, {
    b = changed and lines or nil,
    k = rawm,
    j = { i = jidx, n = jlen, e = jraw },
    g = { i = cidx, n = clen, e = craw },
    p = { pos[1], pos[2], view.curswant },
    buf = bufsym(vim.api.nvim_get_current_buf()),
  })

  -- The rendered listings, for the cases that ask.  They are a
  -- DIFFERENT surface over the same slots (show.rs / jumplist.rs's
  -- printers rather than builtins.rs's getters), so a mutation that
  -- moves one and not the other shows up as a disagreement between the
  -- `K`/`J`/`G` lines and these.
  for _, cmd in ipairs(opt.list or {}) do
    local out, err = capture('silent! ' .. cmd)
    if err then
      emit(label, 'X!', esc(err))
    else
      for i, line in ipairs(vim.split(out, '\n', { plain = true })) do
        emit(label, 'X', string.format('%s|%02d|%s', cmd:gsub('%s+', '_'), i, esc(line)))
      end
      struct(label .. ' X ' .. cmd, vim.split(out, '\n', { plain = true }))
    end
  end
end

--- One case: reset, feed, report.
local function case(label, opt)
  opt = opt or {}
  reset(opt.lines, opt.pos, opt.set)
  if opt.pre then
    local err = quiet(opt.pre)
    if err then
      emit(label, '!pre', esc(err))
    end
  end
  if opt.keys and opt.keys ~= '' then
    local err = feed(opt.keys)
    if err then
      emit(label, '!', esc(err))
    end
  end
  if opt.ex then
    local err = quiet(opt.ex)
    if err then
      emit(label, '!ex', esc(err))
    end
  end
  if opt.keys2 and opt.keys2 ~= '' then
    local err = feed(opt.keys2)
    if err then
      emit(label, '!2', esc(err))
    end
  end
  shot(label, opt)
  if opt.after then
    local ok, res = pcall(vim.fn.eval, 'string(' .. opt.after .. ')')
    emit(label, 'A', ok and esc(scrub(res)) or esc(errtext(res)))
    struct(label .. ' A', ok and scrub(res) or { err = errtext(res) })
  end
end

local SECTIONS = {}
local function section(name, fn)
  SECTIONS[#SECTIONS + 1] = { name = name, fn = fn }
end

-- ---------------------------------------------------------------------
-- s01 -- the lower-case marks
-- ---------------------------------------------------------------------
section('s01-local', function()
  for _, name in ipairs(LOWER) do
    case('s01-set-' .. name, {
      pos = { 5, 3 },
      keys = 'm' .. name,
      after = string.format('[getpos("\'%s"), line("\'%s"), col("\'%s")]', name, name, name),
    })
  end
  -- Set several, then read them all.
  case('s01-many', { keys = '2Gma5Gmb9Gmc13Gmd', list = { 'marks' } })
  -- Re-setting a mark moves it.
  case('s01-reset', { keys = '2Gma9Gma' })
  -- A mark at a column, and at the end of a line.
  case('s01-col', { pos = { 4, 0 }, keys = '5|ma' })
  case('s01-eol', { pos = { 4, 0 }, keys = '$ma' })
  case('s01-empty-line', { lines = { 'a', '', 'c' }, pos = { 2, 0 }, keys = 'ma' })
  -- `m` with a name that is not a mark.
  for _, bad in ipairs({ '1', '#', ' ', '%' }) do
    case('s01-bad-' .. (bad == ' ' and 'space' or bad), { keys = 'm' .. bad })
  end
  -- The uppercase-through-`m` path: `mA` sets a GLOBAL mark even in an
  -- unnamed buffer.
  case('s01-mA-unnamed', { keys = '7GmA', after = 'getpos("\'A")' })
end)

-- ---------------------------------------------------------------------
-- s02 -- global marks across files
-- ---------------------------------------------------------------------
section('s02-global', function()
  -- Three real files, so `'A` has somewhere to go and :marks has a
  -- name to print.
  local open3 = table.concat({
    'silent! edit ' .. work .. '/m1.txt',
    'silent! edit ' .. work .. '/m2.txt',
    'silent! edit ' .. work .. '/m3.txt',
  }, ' | ')
  case('s02-set-three', {
    pre = open3,
    ex = 'silent! buffer m1.txt | 3mark A | silent! buffer m2.txt | 5mark B | silent! buffer m3.txt | 7mark C',
    after = '[getpos("\'A"), getpos("\'B"), getpos("\'C")]',
    list = { 'marks' },
  })
  case('s02-jump-A', {
    pre = open3,
    ex = 'silent! buffer m1.txt | 3mark A | silent! buffer m3.txt',
    keys2 = "'A",
    list = { 'marks', 'jumps' },
  })
  case('s02-jump-backtick-A', {
    pre = open3,
    ex = 'silent! buffer m1.txt | normal! 3G5|mA',
    keys2 = 'G`A',
    list = { 'jumps' },
  })
  -- Every uppercase name, all in one buffer.
  case('s02-all-upper', {
    pre = open3,
    ex = 'silent! buffer m1.txt',
    keys2 = table.concat((function()
      local t = {}
      for i, name in ipairs(UPPER) do
        t[#t + 1] = tostring(((i - 1) % 8) + 1) .. 'Gm' .. name
      end
      return t
    end)(), ''),
    list = { 'marks' },
  })
  -- A global mark re-set in a different buffer moves buffers.
  case('s02-move-buffers', {
    pre = open3,
    ex = 'silent! buffer m1.txt | 2mark A | silent! buffer m2.txt | 6mark A',
    after = 'getpos("\'A")',
    list = { 'marks' },
  })
  -- :mark and :k, the two Ex spellings.
  case('s02-ex-mark', { ex = '4mark a | 6k b', after = '[getpos("\'a"), getpos("\'b")]' })
  case('s02-ex-mark-bad', { ex = 'silent! 4mark ab' })
end)

-- ---------------------------------------------------------------------
-- s03 -- the numbered marks
-- ---------------------------------------------------------------------
section('s03-numbered', function()
  for _, d in ipairs(DIGITS) do
    case('s03-setpos-' .. d, {
      ex = string.format('call setpos("\'%s", [0, %d, 2, 0])', d, tonumber(d) + 1),
      after = string.format('getpos("\'%s")', d),
    })
  end
  case('s03-all-digits', {
    ex = table.concat((function()
      local t = {}
      for i, d in ipairs(DIGITS) do
        t[#t + 1] = string.format('call setpos("\'%s", [0, %d, %d, 0])', d, i, i)
      end
      return t
    end)(), ' | '),
    list = { 'marks', 'marks 0123456789' },
  })
  case('s03-delmarks-digit', {
    ex = 'call setpos("\'0", [0, 3, 1, 0]) | call setpos("\'1", [0, 5, 1, 0]) | delmarks 0',
    list = { 'marks' },
  })
end)

-- ---------------------------------------------------------------------
-- s04 -- the tick family
-- ---------------------------------------------------------------------
section('s04-ticks', function()
  -- '' and `` -- the previous-context mark.
  case('s04-pcmark-G', { keys = 'G' })
  case('s04-pcmark-back', { keys = "G''" })
  case('s04-pcmark-backtick', { keys = 'G``' })
  case('s04-pcmark-twice', { keys = "G''''" })
  case('s04-pcmark-search', { keys = '/l09<CR>', list = { 'jumps' } })
  case('s04-pcmark-colon', { ex = '12', after = 'getpos("\'\'")' })
  case('s04-pcmark-percent', { lines = { 'a(', 'b', 'c)' }, pos = { 1, 1 }, keys = '%' })
  case('s04-pcmark-brace', { keys = '}' })
  -- '[ and '] after each kind of change.
  local ops = {
    { 'yank-3', '3yy' },
    { 'delete-2', '2dd' },
    { 'put', 'yyp' },
    { 'change-word', 'cwXX<Esc>' },
    { 'insert', 'iAB<Esc>' },
    { 'open', 'oNEW<Esc>' },
    { 'join', 'J' },
    { 'shift', '>>' },
    { 'replace', 'rZ' },
    { 'visual-yank', 'Vjy' },
    { 'block-yank', '<C-v>jly' },
    { 'undo', 'ddu' },
    { 'redo', 'ddu<C-r>' },
  }
  for _, o in ipairs(ops) do
    case('s04-brackets-' .. o[1], { pos = { 5, 0 }, keys = o[2] })
  end
  -- '< and '> after each visual shape, forwards and backwards.
  local vis = {
    { 'v-fwd', 'v2j3l<Esc>' },
    { 'v-back', '2j3lv2k<Esc>' },
    { 'V-fwd', 'V2j<Esc>' },
    { 'blk', '<C-v>2j3l<Esc>' },
    { 'blk-dollar', '<C-v>2j$<Esc>' },
    { 'gv', 'v2j<Esc>gv<Esc>' },
    { 'o-swap', 'v2jo<Esc>' },
  }
  for _, v in ipairs(vis) do
    case('s04-visual-' .. v[1], { pos = { 5, 2 }, keys = v[2], list = { 'marks' } })
  end
  -- '^ (last insert), '. (last change), '" (last cursor).
  case('s04-caret', { pos = { 4, 2 }, keys = 'iX<Esc>G', after = '[getpos("\'^"), getpos("\'.")]' })
  case('s04-gi', { pos = { 4, 2 }, keys = 'iX<Esc>Ggi<Esc>' })
  case('s04-dot-after-undo', { pos = { 4, 0 }, keys = 'ddu' })
  -- '( ') '{ '} -- sentence and paragraph, computed from the cursor.
  case('s04-sentence', {
    lines = { 'One two. Three four. Five six.', '', 'Another para.' },
    pos = { 1, 12 },
    after = '[getpos("\'("), getpos("\')"), getpos("\'{"), getpos("\'}")]',
  })
  case('s04-para', {
    lines = { 'a', 'b', '', 'c', 'd', '', 'e' },
    pos = { 4, 0 },
    after = '[getpos("\'{"), getpos("\'}")]',
  })
end)

-- ---------------------------------------------------------------------
-- s05 -- :marks and :delmarks
-- ---------------------------------------------------------------------
section('s05-show', function()
  local build = '2Gma5Gmb9Gmc13GmA3GmB'
  case('s05-marks-all', { keys = build, list = { 'marks' } })
  case('s05-marks-arg-a', { keys = build, list = { 'marks a' } })
  case('s05-marks-arg-abc', { keys = build, list = { 'marks abc' } })
  case('s05-marks-arg-upper', { keys = build, list = { 'marks AB' } })
  case('s05-marks-arg-mixed', { keys = build, list = { 'marks aAb' } })
  case('s05-marks-arg-tick', { keys = build .. 'G', list = { "marks '" } })
  case('s05-marks-arg-brackets', { keys = build .. 'yy', list = { 'marks []' } })
  case('s05-marks-arg-none-set', { list = { 'marks' } })
  case('s05-marks-arg-unknown', { keys = build, list = { 'marks z' } })
  case('s05-marks-long-text', {
    lines = { string.rep('x', 200), 'b', 'c' },
    keys = 'ma',
    list = { 'marks' },
  })
  case('s05-marks-leading-ws', {
    lines = { '\t\t  indented mark line', 'b' },
    keys = 'ma',
    list = { 'marks' },
  })
  -- :delmarks in every shape.
  for _, arg in ipairs({ 'a', 'a b', 'a-c', 'A', 'A-C', '0', 'a-cA-C', '!', 'z', '"', "'" }) do
    case('s05-delmarks-' .. arg:gsub('%W', function(c)
      return string.format('x%02x', c:byte())
    end), {
      keys = build .. 'yyG',
      ex = 'silent! delmarks ' .. arg,
      list = { 'marks' },
    })
  end
  case('s05-delmarks-noarg', { keys = build, ex = 'silent! delmarks', list = { 'marks' } })
  case('s05-delmarks-bang-arg', { keys = build, ex = 'silent! delmarks! a', list = { 'marks' } })
end)

-- ---------------------------------------------------------------------
-- s06 -- the jumplist
-- ---------------------------------------------------------------------
section('s06-jumps', function()
  local pushers = {
    { 'G', 'G' },
    { 'gg', 'Ggg' },
    { 'count-G', '9G' },
    { 'search', '/l12<CR>' },
    { 'search-back', 'G?l03<CR>' },
    { 'star', '5G*' },
    { 'brace', '}' },
    { 'paren', ')' },
    { 'H-L', 'HL' },
    { 'pct', '50%' },
    { 'mark-jump', '9Gma1G' .. "'a" },
    { 'backtick-jump', '9Gma1G`a' },
    { 'tick-tick', "G''" },
    { 'gd-like', '5GG5G' },
  }
  for _, p in ipairs(pushers) do
    case('s06-push-' .. p[1], { keys = p[2], list = { 'jumps' } })
  end
  -- <C-o> / <C-i> walk, and the index it leaves behind.
  case('s06-ctrl-o', { keys = '3G9G14G<C-o>', list = { 'jumps' } })
  case('s06-ctrl-o-2', { keys = '3G9G14G<C-o><C-o>', list = { 'jumps' } })
  case('s06-ctrl-o-past-end', { keys = '3G9G<C-o><C-o><C-o><C-o>', list = { 'jumps' } })
  case('s06-ctrl-i', { keys = '3G9G14G<C-o><C-o><C-i>', list = { 'jumps' } })
  case('s06-ctrl-i-past-end', { keys = '3G9G14G<C-i><C-i>', list = { 'jumps' } })
  case('s06-jump-after-ctrl-o', { keys = '3G9G14G<C-o>6G', list = { 'jumps' } })
  -- 'jumpoptions'.
  for _, jo in ipairs({ '', 'stack', 'view', 'stack,view', 'clean' }) do
    case('s06-jopt-' .. (jo == '' and 'empty' or jo:gsub('%W', '_')), {
      set = 'set jumpoptions=' .. jo,
      keys = '3G9G14G<C-o>6G',
      list = { 'jumps' },
    })
  end
  -- The dedup: two jumps from the same line.
  case('s06-dedup-same-line', { keys = '5G9G5G9G5G', list = { 'jumps' } })
  case('s06-dedup-after-delete', { keys = '5G9G12G', ex = '5,9delete', list = { 'jumps' } })
  -- :clearjumps.
  case('s06-clearjumps', { keys = '3G9G14G', ex = 'clearjumps', list = { 'jumps' } })
  case('s06-clearjumps-then-jump', { keys = '3G9G', ex = 'clearjumps', keys2 = '14G', list = { 'jumps' } })
  -- Past JUMPLISTSIZE.  130 distinct jumps into a 130-line buffer: the
  -- list saturates and the OLDEST entries fall off, so `i`, `n` and
  -- WHICH lines survive are all part of the answer.
  local big = {}
  for i = 1, 130 do
    big[i] = string.format('b%03d', i)
  end
  local walk = {}
  for i = 1, 130 do
    walk[#walk + 1] = tostring(i) .. 'G'
  end
  case('s06-overflow', { lines = big, keys = table.concat(walk, ''), list = { 'jumps' } })
  case('s06-overflow-ctrl-o', {
    lines = big,
    keys = table.concat(walk, '') .. '<C-o><C-o><C-o>',
    list = { 'jumps' },
  })
  -- A jump into another buffer, and back.
  case('s06-cross-buffer', {
    pre = 'silent! edit ' .. work .. '/m1.txt',
    ex = 'silent! 4mark A | silent! buffer main.txt',
    keys2 = "9G'A",
    list = { 'jumps' },
  })
end)

-- ---------------------------------------------------------------------
-- s07 -- the changelist
-- ---------------------------------------------------------------------
section('s07-changes', function()
  local changers = {
    { 'x', 'x' },
    { 'dd', 'dd' },
    { 'insert', 'iA<Esc>' },
    { 'append', 'aB<Esc>' },
    { 'open', 'oC<Esc>' },
    { 'replace', 'rZ' },
    { 'change-word', 'cwQQ<Esc>' },
    { 'put', 'yyp' },
    { 'join', 'J' },
    { 'shift', '>>' },
    { 'undo', 'xu' },
    { 'redo', 'xu<C-r>' },
  }
  for _, c in ipairs(changers) do
    case('s07-' .. c[1], { pos = { 5, 2 }, keys = c[2], list = { 'changes' } })
  end
  -- Several changes on different lines, then g; and g, walk them.
  local build = '2Gx6Gx10Gx14Gx'
  case('s07-build', { keys = build, list = { 'changes' } })
  case('s07-gsemi', { keys = build .. 'g;', list = { 'changes' } })
  case('s07-gsemi-2', { keys = build .. 'g;g;', list = { 'changes' } })
  case('s07-gsemi-past', { keys = build .. 'g;g;g;g;g;g;', list = { 'changes' } })
  case('s07-gcomma', { keys = build .. 'g;g;g,', list = { 'changes' } })
  case('s07-gcomma-past', { keys = build .. 'g;g,g,g,', list = { 'changes' } })
  case('s07-count-gsemi', { keys = build .. '3g;', list = { 'changes' } })
  -- The same-line dedup: two changes on one line collapse.
  case('s07-dedup-same-line', { pos = { 5, 0 }, keys = 'xxx', list = { 'changes' } })
  case('s07-dedup-far-col', {
    lines = { 'l01', string.rep('y', 200), 'l03' },
    pos = { 2, 0 },
    keys = 'x150|x',
    list = { 'changes' },
  })
  -- Past JUMPLISTSIZE: the changelist is the same fixed 100-entry
  -- array and shifts the same way.
  local big = {}
  for i = 1, 130 do
    big[i] = string.format('c%03d', i)
  end
  local walk = {}
  for i = 1, 130 do
    walk[#walk + 1] = tostring(i) .. 'Gx'
  end
  case('s07-overflow', { lines = big, keys = table.concat(walk, ''), list = { 'changes' } })
  case('s07-overflow-gsemi', {
    lines = big,
    keys = table.concat(walk, '') .. 'g;g;g;',
    list = { 'changes' },
  })
  -- The changelist survives a jumplist clear, and vice versa.
  case('s07-independent', { keys = build .. '2G9G', ex = 'clearjumps', list = { 'changes', 'jumps' } })
end)

-- ---------------------------------------------------------------------
-- s08 -- adjustment
-- ---------------------------------------------------------------------
section('s08-adjust', function()
  -- Marks at 3, 6, 9, 12 plus a global at 6 and a jumplist and a
  -- changelist, so ONE case answers what every store did about one
  -- edit.
  local build = '3Gma6Gmb9Gmc12Gmd6GmA3G6G9G12G1G'
  local edits = {
    { 'dd-above', '1Gdd' },
    { 'dd-at-a', '3Gdd' },
    { 'dd-between', '5Gdd' },
    { 'dd-at-d', '12Gdd' },
    { 'dd-below', '15Gdd' },
    { 'dd3-across-a', '2G3dd' },
    { 'dd3-across-b', '5G3dd' },
    { 'ddG', '6GdG' },
    { 'dgg', '9Gdgg' },
    { 'o-above', '1GoNEW<Esc>' },
    { 'o-between', '5GoNEW<Esc>' },
    { 'O-at-b', '6GONEW<Esc>' },
    { 'p-lines', '1Gyy5Gp' },
    { 'P-lines', '1Gyy5GP' },
    { 'J-at-b', '6GJ' },
    { 'J3', '5G3J' },
    { 'shift-right', '6G>>' },
    { 'shift-left', '6G<<<<' },
    { 'x-before-col', '6G0x' },
    { 'sub-split', ':6s/word06/A\\rB/<CR>' },
    { 'sub-join-all', ':%s/\\n/ /<CR>' },
    { 'undo-dd', '3Gddu' },
    { 'redo-dd', '3Gddu<C-r>' },
  }
  for _, e in ipairs(edits) do
    case('s08-' .. e[1], { keys = build .. e[2], list = { 'marks', 'jumps', 'changes' } })
  end
  local excmds = {
    { 'move-down', '3move 10' },
    { 'move-up', '10move 2' },
    { 'move-range', '3,6move 12' },
    { 'move-to-0', '9move 0' },
    { 'move-to-end', '3move $' },
    { 'copy', '3,4copy 10' },
    { 'delete-range', '4,8delete' },
    { 'put', '3yank | 8put' },
    { 'global-del', 'g/l0[369]/delete' },
    { 'global-move', 'g/l1[012]/move 0' },
    { 'normal-dd', '5normal! dd' },
    { 'join-range', '5,8join' },
    { 'sort', 'sort!' },
    { 'retab', 'set expandtab | 6s/^/\\t\\t/ | retab' },
  }
  for _, e in ipairs(excmds) do
    case('s08-ex-' .. e[1], { keys = build, ex = 'silent! ' .. e[2], list = { 'marks' } })
  end
  -- The three stores nothing above touches, because only an INSERT or
  -- a CHANGE sets them: `'^` (b_last_insert), `'.` (b_last_change) and
  -- `'"` (b_last_cursor).  `mark_adjust_buf` shifts each of the eight
  -- per-buffer fmarks by hand, with `line` for some and `line_nodel`
  -- for others (invalidate vs land-on-the-first-deleted-line), and
  -- WHICH variant each one gets is a per-field decision no other case
  -- here reaches -- `1787242636-jmarkmutate.py`'s `mark-caret-nodel`
  -- was measured NOT CAUGHT until this block existed.
  local ins = '6GiX<Esc>1G'
  for _, e in ipairs({
    { 'dd-at-caret', '6Gdd' },
    { 'dd-range-over-caret', '5G3dd' },
    { 'dd-above-caret', '2Gdd' },
    { 'dd-below-caret', '9Gdd' },
    { 'dG-from-caret', '6GdG' },
    { 'o-above-caret', '5GoNEW<Esc>' },
  }) do
    case('s08-ins-' .. e[1], {
      keys = ins .. e[2],
      after = '[getpos("\'^"), getpos("\'."), getpos("\'\\"")]',
    })
  end
  for _, e in ipairs({
    { 'move-caret-down', '6move 12' },
    { 'move-over-caret', '4,8move 14' },
    { 'delete-caret', '5,7delete' },
    { 'put-above-caret', '1yank | 3put' },
  }) do
    case('s08-ins-ex-' .. e[1], {
      keys = ins,
      ex = 'silent! ' .. e[2],
      after = '[getpos("\'^"), getpos("\'."), getpos("\'\\"")]',
    })
  end

  -- Column adjustment specifically: a mark mid-line, then an edit
  -- before it, at it, and after it.
  for _, e in ipairs({ { 'before', '0x' }, { 'at', '5|x' }, { 'after', '$x' }, { 'ins-before', '0iZZ<Esc>' } }) do
    case('s08-col-' .. e[1], { pos = { 5, 4 }, keys = 'ma' .. e[2], after = 'getpos("\'a")' })
  end
end)

-- ---------------------------------------------------------------------
-- s09 -- marks in a buffer that goes away
-- ---------------------------------------------------------------------
section('s09-gone', function()
  local prep = 'silent! edit ' .. work .. '/m1.txt'
  case('s09-bunload', {
    pre = prep,
    ex = 'silent! 4mark A | silent! 5mark a | silent! buffer main.txt | silent! bunload! m1.txt',
    after = '[getpos("\'A"), bufloaded("m1.txt")]',
    list = { 'marks' },
  })
  case('s09-bdelete', {
    pre = prep,
    ex = 'silent! 4mark A | silent! buffer main.txt | silent! bdelete! m1.txt',
    after = '[getpos("\'A"), bufexists("m1.txt")]',
    list = { 'marks' },
  })
  case('s09-bwipe', {
    pre = prep,
    ex = 'silent! 4mark A | silent! buffer main.txt | silent! bwipeout! m1.txt',
    after = '[getpos("\'A"), bufexists("m1.txt")]',
    list = { 'marks' },
  })
  case('s09-jump-to-wiped', {
    pre = prep,
    ex = 'silent! 4mark A | silent! buffer main.txt | silent! bwipeout! m1.txt',
    keys2 = "'A",
    list = { 'marks', 'jumps' },
  })
  case('s09-jumplist-wiped', {
    pre = prep,
    ex = 'silent! normal! 4G | silent! buffer main.txt | silent! normal! 9G | silent! bwipeout! m1.txt',
    list = { 'jumps' },
  })
  case('s09-local-marks-after-bwipe', {
    pre = prep,
    ex = 'silent! 4mark a | silent! bwipeout! m1.txt',
    after = 'getpos("\'a")',
  })
end)

-- ---------------------------------------------------------------------
-- s10 -- the shada round trip
-- ---------------------------------------------------------------------
section('s10-shada', function()
  -- The persisted BYTES are perssweep's golden; what this asks is what
  -- comes BACK.  One fixed shada file under $WORK, rewritten per case,
  -- so nothing here depends on the caller's real shada.
  local sf = work .. '/j.shada'
  local function trip(label, setup, opts)
    reset(nil, { 1, 0 }, nil)
    quiet('silent! edit ' .. work .. '/m1.txt')
    quiet(setup)
    os.remove(sf)
    quiet('silent! wshada! ' .. sf)
    -- Wipe every mark this process holds, then read it back.  A
    -- surviving in-memory mark would make the round trip look like it
    -- worked when it did nothing.
    quiet('silent! buffer main.txt')
    clearmarks()
    quiet('silent! bwipeout! m1.txt')
    quiet('silent! rshada! ' .. sf)
    scanbufs()
    shot(label, opts or { list = { 'marks' } })
  end
  trip('s10-globals', 'silent! 3mark A | silent! 5mark B | silent! 7mark C')
  trip('s10-numbered', 'call setpos("\'0", [0, 2, 1, 0]) | call setpos("\'1", [0, 4, 1, 0])')
  trip('s10-locals-only', 'silent! 3mark a | silent! 5mark b')
  trip('s10-jumplist', 'silent! normal! 3G9G12G', { list = { 'jumps', 'marks' } })
  trip('s10-changelist', 'silent! normal! 3Gx9Gx', { list = { 'changes', 'marks' } })
  trip('s10-visual', 'silent! normal! 3GVjj<Esc>', { list = { 'marks' } })
  -- A merge: two writes, the second one older in content but newer in
  -- time, so `mark_set_global`'s timestamp comparison decides.
  reset(nil, { 1, 0 }, nil)
  quiet('silent! edit ' .. work .. '/m1.txt')
  quiet('silent! 3mark A')
  os.remove(sf)
  quiet('silent! wshada! ' .. sf)
  quiet('silent! 9mark A')
  quiet('silent! wshada ' .. sf) -- merging write, not `!`
  quiet('silent! buffer main.txt')
  clearmarks()
  quiet('silent! bwipeout! m1.txt')
  quiet('silent! rshada! ' .. sf)
  scanbufs()
  shot('s10-merge', { list = { 'marks' } })
  os.remove(sf)
end)

-- ---------------------------------------------------------------------
-- s11 -- setpos / getpos
-- ---------------------------------------------------------------------
section('s11-setpos', function()
  local probes = {
    { "setpos(\"'a\", [0, 5, 3, 0])", 'a-plain' },
    { "setpos(\"'a\", [0, 0, 3, 0])", 'a-line0' },
    { "setpos(\"'a\", [0, 999, 3, 0])", 'a-past-end' },
    { "setpos(\"'a\", [0, -1, 3, 0])", 'a-neg-line' },
    { "setpos(\"'a\", [0, 5, 0, 0])", 'a-col0' },
    { "setpos(\"'a\", [0, 5, 999, 0])", 'a-past-col' },
    { "setpos(\"'a\", [0, 5, 3, 7])", 'a-off' },
    { "setpos(\"'a\", [0, 5, 3])", 'a-short' },
    { "setpos(\"'a\", [0, 5, 3, 0, 9])", 'a-long' },
    { "setpos(\"'a\", 'nope')", 'a-notlist' },
    { "setpos(\"'A\", [0, 5, 3, 0])", 'A-plain' },
    { "setpos(\"'<\", [0, 3, 2, 0])", 'lt' },
    { "setpos(\"'>\", [0, 7, 4, 0])", 'gt' },
    { "setpos(\"'[\", [0, 3, 2, 0])", 'bro' },
    { "setpos(\"']\", [0, 7, 4, 0])", 'brc' },
    { "setpos(\"'^\", [0, 4, 2, 0])", 'caret' },
    { "setpos(\"'.\", [0, 4, 2, 0])", 'dot' },
    { "setpos(\"''\", [0, 4, 2, 0])", 'tick' },
    { "setpos('.', [0, 6, 4, 0])", 'cursor' },
    { "setpos('.', [0, 6, 4, 3])", 'cursor-off' },
    { "setpos('x', [0, 6, 4, 0])", 'badname' },
    { "setpos(\"'1\", [0, 6, 4, 0])", 'digit' },
  }
  for _, p in ipairs(probes) do
    case('s11-' .. p[2], {
      after = string.format(
        "[%s, getpos(\"'a\"), getpos(\"'A\"), getpos(\"'<\"), getpos(\"'>\"), getpos(\"'[\"), getpos(\"']\"), getpos('.')]",
        p[1]
      ),
      ex = 'call ' .. p[1]:gsub('^', '') .. ' | echo ""',
    })
  end
  -- getpos / getcurpos / line / col / virtcol over one mark.
  case('s11-readers', {
    pos = { 5, 4 },
    keys = 'ma',
    after = "[getpos(\"'a\"), line(\"'a\"), col(\"'a\"), virtcol(\"'a\"), getcharpos(\"'a\")]",
  })
  case('s11-readers-unset', {
    after = "[getpos(\"'z\"), line(\"'z\"), col(\"'z\"), virtcol(\"'z\")]",
  })
  case('s11-getcurpos', { pos = { 7, 3 }, after = 'getcurpos()' })
  case('s11-setcharpos', {
    ex = 'call setcharpos("\'a", [0, 5, 3, 0])',
    after = "[getpos(\"'a\"), getcharpos(\"'a\")]",
  })
  -- Multibyte: getpos is bytes, getcharpos is characters.
  case('s11-multibyte', {
    lines = { 'aaa', 'ααββγγ', 'ccc' },
    pos = { 2, 4 },
    keys = 'ma',
    after = "[getpos(\"'a\"), getcharpos(\"'a\"), col(\"'a\"), virtcol(\"'a\")]",
  })
end)

-- ---------------------------------------------------------------------
-- s12 -- the API surface
-- ---------------------------------------------------------------------
section('s12-api', function()
  local function api(label, fn)
    reset(nil, { 1, 0 }, nil)
    local ok, res = pcall(fn)
    emit(label, 'A', ok and esc(scrub(vim.inspect(res):gsub('%s+', ' '))) or esc(errtext(res)))
    struct(label .. ' A', ok and res or { err = errtext(res) })
    shot(label)
  end
  api('s12-buf-set-mark', function()
    return {
      vim.api.nvim_buf_set_mark(0, 'a', 5, 3, {}),
      vim.api.nvim_buf_get_mark(0, 'a'),
    }
  end)
  api('s12-buf-set-mark-upper', function()
    return {
      vim.api.nvim_buf_set_mark(0, 'A', 6, 2, {}),
      vim.api.nvim_get_mark('A', {}),
    }
  end)
  api('s12-buf-set-mark-bad', function()
    return vim.api.nvim_buf_set_mark(0, 'ab', 5, 3, {})
  end)
  api('s12-buf-set-mark-line0', function()
    return vim.api.nvim_buf_set_mark(0, 'a', 0, 0, {})
  end)
  api('s12-buf-set-mark-past-end', function()
    return vim.api.nvim_buf_set_mark(0, 'a', 999, 0, {})
  end)
  api('s12-buf-del-mark', function()
    vim.api.nvim_buf_set_mark(0, 'a', 5, 3, {})
    return { vim.api.nvim_buf_del_mark(0, 'a'), vim.api.nvim_buf_get_mark(0, 'a') }
  end)
  api('s12-buf-del-mark-unset', function()
    return vim.api.nvim_buf_del_mark(0, 'q')
  end)
  api('s12-del-mark', function()
    vim.api.nvim_buf_set_mark(0, 'A', 5, 3, {})
    return { vim.api.nvim_del_mark('A'), vim.api.nvim_get_mark('A', {}) }
  end)
  api('s12-del-mark-lower', function()
    return vim.api.nvim_del_mark('a')
  end)
  api('s12-get-mark-unset', function()
    return vim.api.nvim_get_mark('Z', {})
  end)
  api('s12-buf-get-mark-ticks', function()
    vim.api.nvim_win_set_cursor(0, { 4, 2 })
    vim.api.nvim_feedkeys(vim.api.nvim_replace_termcodes('Vjy', true, true, true), 'ntx', false)
    return {
      vim.api.nvim_buf_get_mark(0, '<'),
      vim.api.nvim_buf_get_mark(0, '>'),
      vim.api.nvim_buf_get_mark(0, '['),
      vim.api.nvim_buf_get_mark(0, ']'),
    }
  end)
  api('s12-getmarklist-buf', function()
    vim.api.nvim_buf_set_mark(0, 'a', 5, 3, {})
    vim.api.nvim_buf_set_mark(0, 'b', 7, 1, {})
    return vim.fn.getmarklist(vim.api.nvim_get_current_buf())
  end)
  api('s12-getmarklist-global', function()
    vim.api.nvim_buf_set_mark(0, 'A', 5, 3, {})
    return vim.fn.getmarklist()
  end)
end)

-- ---------------------------------------------------------------------
-- s13 -- the motions
-- ---------------------------------------------------------------------
section('s13-motion', function()
  local fixture = { 'l01', '    l02 indented', 'l03', '        l04 deeper', 'l05', 'l06', 'l07', 'l08' }
  case('s13-tick-linewise', { lines = fixture, pos = { 2, 8 }, keys = 'ma1G' .. "'a" })
  case('s13-backtick-exact', { lines = fixture, pos = { 2, 8 }, keys = 'ma1G`a' })
  case('s13-tick-first-nonblank', { lines = fixture, pos = { 4, 12 }, keys = 'ma1G' .. "'a" })
  case('s13-g-tick', { lines = fixture, pos = { 2, 8 }, keys = "ma1Gg'a", list = { 'jumps' } })
  case('s13-g-backtick', { lines = fixture, pos = { 2, 8 }, keys = 'ma1Gg`a', list = { 'jumps' } })
  case('s13-tick-pushes-jump', { lines = fixture, pos = { 2, 8 }, keys = "ma1G'a", list = { 'jumps' } })
  -- Operators over a mark motion, and what '[ '] answer afterwards.
  case('s13-op-tick', { lines = fixture, pos = { 6, 0 }, keys = "ma2Gd'a" })
  case('s13-op-backtick', { lines = fixture, pos = { 6, 2 }, keys = 'ma2Gd`a' })
  case('s13-op-y-tick', { lines = fixture, pos = { 6, 0 }, keys = "ma2Gy'a" })
  -- The error arms: an unset mark, an invalid name, a mark whose line
  -- was deleted.
  case('s13-E20-unset', { keys = "'z" })
  case('s13-E78-badname', { keys = "'&" })
  case('s13-E20-deleted', { keys = '5Gma1G5Gdd1G' .. "'a" })
  case('s13-mark-in-other-buffer', {
    pre = 'silent! edit ' .. work .. '/m1.txt',
    ex = 'silent! 4mark A | silent! buffer main.txt',
    keys2 = "'A",
    list = { 'jumps' },
  })
  -- getnextmark is reached by `]'` / `['` / `]` ` / `[` `.
  for _, k in ipairs({ "]'", "['", ']`', '[`' }) do
    case('s13-next-' .. k:gsub('%W', function(c)
      return string.format('x%02x', c:byte())
    end), {
      keys = '3Gma7Gmb11Gmc5G' .. k,
    })
    case('s13-next-count-' .. k:gsub('%W', function(c)
      return string.format('x%02x', c:byte())
    end), {
      keys = '3Gma7Gmb11Gmc1G2' .. k,
    })
  end
end)

-- ---------------------------------------------------------------------
-- s14 -- getmarklist() beside :marks
-- ---------------------------------------------------------------------
section('s14-getmarklist', function()
  local build = '2Gma5Gmb9GmA13GmB'
  case('s14-both', { keys = build, after = '[getmarklist(bufnr("%")), getmarklist()]', list = { 'marks' } })
  case('s14-buf-only', { keys = build, after = 'getmarklist(bufnr("%"))' })
  case('s14-global-only', { keys = build, after = 'getmarklist()' })
  case('s14-empty', { after = '[getmarklist(bufnr("%")), getmarklist()]' })
  case('s14-bad-arg', { after = 'getmarklist(99999)' })
  case('s14-with-ticks', { keys = build .. 'yyG', after = 'getmarklist(bufnr("%"))', list = { 'marks' } })
  case('s14-cols', {
    lines = { 'aaa', 'ααββγγ', 'ccc' },
    pos = { 2, 4 },
    keys = 'ma',
    after = '[getmarklist(bufnr("%")), getpos("\'a")]',
  })
end)

-- ---------------------------------------------------------------------
-- s15 -- :lockmarks
-- ---------------------------------------------------------------------
section('s15-lockmarks', function()
  local build = '3Gma6Gmb9Gmc12Gmd6GmA1G'
  local cmds = {
    { 'move', '3move 10' },
    { 'delete', '4,8delete' },
    { 'copy', '3,4copy 10' },
    { 'normal-dd', '5normal! dd' },
    { 'put', '3yank | 8put' },
    { 'global-del', 'g/l0[369]/delete' },
  }
  for _, c in ipairs(cmds) do
    case('s15-locked-' .. c[1], { keys = build, ex = 'silent! lockmarks ' .. c[2], list = { 'marks' } })
    case('s15-plain-' .. c[1], { keys = build, ex = 'silent! ' .. c[2], list = { 'marks' } })
  end
  case('s15-keepjumps', { keys = build, ex = 'silent! keepjumps normal! G', list = { 'jumps' } })
  -- `:keepmarks` over the internal commands only.  It used to be
  -- `3,6!cat` here, which SHELLS OUT: `env -i PATH=/usr/bin:/bin` has
  -- no `cat` on a NixOS host, so the fixture picked up
  -- `/bin/sh: line 1: cat: command not found` and the artifact became
  -- a function of the machine's filesystem layout.  No case in this
  -- sweep may run an external program.
  case('s15-keepmarks-move', { keys = build, ex = 'silent! keepmarks 3,6move 12', list = { 'marks' } })
  case('s15-keepmarks-normal', { keys = build, ex = 'silent! keepmarks 5normal! dd', list = { 'marks' } })
  case('s15-keepmarks-del', { keys = build, ex = 'silent! keepmarks 4,8delete', list = { 'marks' } })

  -- The COLUMN path.  Every command above is a LINE operation, and a
  -- line operation reaches `mark_adjust_buf`, whose `:lockmarks`
  -- short-circuit is a different early return in a different function
  -- from `mark_col_adjust`'s.  p20-12 (trap 12) measured the column one
  -- as UNREACHABLE from this section and left it unanchored; these
  -- cases are what closes that.
  --
  -- `mark_col_adjust`'s only live callers are `ops/join.rs` (J, :join
  -- -- `col_amount > 0` with `lnum_amount = -1` and a non-zero
  -- `spaces_removed`), `change/open_line/mod.rs`'s OPENLINE_MARKFIX (a
  -- substitute that inserts a newline -- `col_amount < 0`,
  -- `lnum_amount = 1`) and `textformat/lines.rs`.  Both arms need a
  -- mark whose COLUMN moves, so these marks are set MID-LINE; a mark
  -- at column 0 is below `mincol` for the join arm and its column
  -- would not move whether the guard fired or not, which is exactly
  -- how the gap went unnoticed.
  --
  -- `'a` sits on line 5 and `'b`/`'A` on line 6 on purpose: a join at
  -- line 5 adjusts only line 6, so one mark that MUST move and one
  -- that must NOT are in the same report line.  `'c` is at the END of
  -- line 5, which is the only one of the four that is at or past the
  -- `mincol` OPENLINE_MARKFIX passes, so it is the substitute arm's
  -- moving mark the way `'b` is the join arm's.
  local colbuild = '5G5|ma5G$mc6G3|mb6G8|mA5G'
  local colcmds = {
    { 'join', 'normal! 5GJ' },
    { 'join-ex', '5,6join' },
    { 'join-count', 'normal! 5G3J' },
    { 'join-nospace', '5,6join!' },
    -- A substitute that inserts a `\r` is here as the CONTROL, not as a
    -- second column case: `open_line` only receives OPENLINE_MARKFIX
    -- from `textformat/wrap.rs`, so this pair moves lines and never
    -- reaches `mark_col_adjust` at all.  Recorded so nobody re-derives
    -- that from the source.
    { 'sub-split', '5s/word05/A\\rB/' },
    { 'sub-split-two', '5,6s/word0/X\\rY/' },
  }
  for _, c in ipairs(colcmds) do
    case('s15-col-locked-' .. c[1], {
      keys = colbuild,
      ex = 'silent! lockmarks ' .. c[2],
      list = { 'marks' },
      after = '[getpos("\'a"), getpos("\'b"), getpos("\'c"), getpos("\'A")]',
    })
    case('s15-col-plain-' .. c[1], {
      keys = colbuild,
      ex = 'silent! ' .. c[2],
      list = { 'marks' },
      after = '[getpos("\'a"), getpos("\'b"), getpos("\'c"), getpos("\'A")]',
    })
  end

  -- `mark_col_adjust`'s other two arms, both reached only through
  -- 'textwidth' formatting: `textformat/wrap.rs` is the sole caller
  -- that passes OPENLINE_MARKFIX (`col_amount < 0`, `lnum_amount = 1`)
  -- and `textformat/lines.rs` strips leading whitespace with
  -- `lnum_amount = 0` and `mincol = 0`.  The mark is at the END of the
  -- long line so it is past every `mincol` these pass.
  local wide = vim.deepcopy(FLAT)
  wide[5] = 'alpha beta gamma delta epsilon zeta eta theta iota kappa'
  for _, c in ipairs({
    { 'gqq', 'normal! 5Ggqq' },
    { 'gq-range', 'normal! 5Ggqj' },
  }) do
    for _, mode in ipairs({ { 'locked', 'lockmarks ' }, { 'plain', '' } }) do
      case('s15-fmt-' .. mode[1] .. '-' .. c[1], {
        lines = wide,
        set = 'setlocal textwidth=12',
        keys = '5G$ma5G8|mb1G',
        ex = 'silent! ' .. mode[2] .. c[2],
        after = '[getpos("\'a"), getpos("\'b")]',
        list = { 'marks' },
      })
    end
  end
end)

-- ---------------------------------------------------------------------
-- s16 -- uncaptured, for the .stderr artifact
-- ---------------------------------------------------------------------
section('s16-messages', function()
  -- nvim's messages carry no trailing newline in a headless process,
  -- so without a separator the whole section arrives as one blob and a
  -- diff can only say "it moved".  The marker names the probe.
  local function mark(kind, what)
    io.stdout:flush()
    io.stderr:write('\n-- s16 ', kind, ' ', what, ': ')
    io.stderr:flush()
  end
  reset(nil, { 1, 0 }, nil)
  quiet('set report=0')
  for _, k in ipairs({ "'z", '`z', "'&", '`&', "']", "'(", '5Gma1G' .. "'a", "1G']", 'g;', 'g,', "]'", "['" }) do
    mark('keys', k)
    feed(k)
  end
  for _, c in ipairs({
    'marks',
    'marks zzz',
    'delmarks',
    'delmarks q-a',
    'delmarks 1-0',
    'delmarks *',
    'delmarks! a',
    'jumps',
    'changes',
    'clearjumps',
    'normal! 3Gma',
    'marks a',
    'call setpos("\'a", [0, 1])',
    'call setpos("\'a", "x")',
    'call setpos("$", [0, 1, 1, 0])',
    'echo getpos("\'q")',
    'echo getmarklist(-1)',
    'echo getjumplist(-1)',
    'echo getchangelist(-1)',
    'echo getjumplist(1, 99)',
  }) do
    mark('ex', c)
    local ok, err = pcall(vim.api.nvim_exec2, c, { output = false })
    if not ok then
      io.stderr:write('EX-ERR ', errtext(err))
    end
  end
  io.stderr:write('\n')
  quiet('set report=9999')
  reset(nil, { 1, 0 }, nil)
  emit('s16-done', 'K', 'ok')
end)

-- ---------------------------------------------------------------------
-- Run.
-- ---------------------------------------------------------------------

quiet('set noswapfile nomore noshowmode shortmess=filnxtToOFS report=9999 belloff=all')
quiet('set encoding=utf-8 fileencoding= isprint=@,161-255 ambiwidth=single')
quiet('set columns=80 lines=24 cmdheight=1 laststatus=0 ruler& showcmd&')
quiet('set hidden nobackup nowritebackup')
quiet('language C')
quiet('syntax off')
quiet('silent! cd ' .. work)

-- Three real files plus a named scratch, written from here so their
-- contents are part of this script rather than of the wrapper.  Named,
-- because the report prints buffer NAMES and a `[No Name]` buffer has
-- only a handle.
for _, name in ipairs({ 'm1.txt', 'm2.txt', 'm3.txt' }) do
  local fd = assert(io.open(work .. '/' .. name, 'w'))
  for i = 1, 16 do
    fd:write(string.format('%s line %02d\n', name:sub(1, 2), i))
  end
  fd:close()
end

quiet('silent! edit ' .. work .. '/main.txt')
BUF = vim.api.nvim_get_current_buf()
BUFNAME[BUF] = 'main.txt'
vim.api.nvim_buf_set_lines(BUF, 0, -1, false, FLAT)
quiet('silent! write!')
quiet(DEFAULTS)
scanbufs()

emit('== defaults ==')
emit(
  'defaults',
  'D',
  string.format(
    'jop=%s sd=%s cols=%d main=%s',
    esc(vim.o.jumpoptions),
    esc(vim.o.shada),
    vim.o.columns,
    bufsym(BUF)
  )
)

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
