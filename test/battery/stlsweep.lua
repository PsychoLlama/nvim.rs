-- stlsweep -- the sixteenth baselined differential.  Driven by
-- stlsweep.sh, which builds the sandbox, pins $HOME/$TMPDIR/
-- $PATH and does the scrubs only the shell can see.  Read that header.
--
-- The subsystem is statusline.rs: `build_stl_str_hl` (1,751 lines, the
-- single biggest item in the whole port) plus `win_redr_custom`,
-- `redraw_ruler`, `draw_tabline` and the click-definition arenas.
-- Before this oracle the whole file had THREE snapshots in all of
-- scrsweep -- 2 `statusline=`, 6 `%{`, and ZERO `%@` -- against 41
-- distinct `STL_*` item letters, groups, min/max width, `%<`, `%=`,
-- `%{%..%}`, `%!` and a hundred-deep evaluation wall (B19 survey S3).
--
-- The design lever is `nvim_eval_statusline()`: it returns
-- `{str, width, highlights}` for an arbitrary format in an arbitrary
-- window context, which is a TEXTUAL answer for the entire alphabet and
-- is both sharper and cheaper than a screen dump.  Screen snapshots are
-- used only where it cannot reach:
--
--   * click definitions (s6) -- `nvim_eval_statusline` parses `%@..%X`
--     but throws the click records away, and the arenas are only filled
--     by a real draw.  They are read back by CLICKING: an `--embed`
--     child driven over RPC, whose handler records its own arguments.
--     A click is only dispatched by the main input loop, so it can NOT
--     be done in this `-l` process -- `nvim_input_mouse` here leaves the
--     key sitting in the typeahead (`getchar(0)` fishes it back out) and
--     no `%@` handler ever runs.
--   * `'rulerformat'` (s8) -- `redraw_ruler` bails when the window it
--     picks has a status line, so with `laststatus=2` it renders only
--     with the cursor parked in a FLOAT (B19-1).  Read off the internal
--     grid with `screenstring()`, in the same child.
--   * the sandbox arm (s5) -- `nvim_eval_statusline` passes
--     `kOptInvalid` as `opt_idx`, so `use_sandbox` is unconditionally
--     false there; only an insecurely-set option (a modeline) reaches
--     it, and only through a real redraw.
--
-- Sections:
--   s1  items        41 item letters x 9 buffer states
--   s2  width        min/max width, precision, the 50 clamp, %N forms
--   s3  groups       %(..%) nesting, empty-group elision, %= sharing
--   s4  trunc        %< at every position, multibyte/double-width cuts
--   s5  eval         %{}, %{%..%}, %!, errors, sandbox, the depth wall
--   s6  click        %@Func@..%X and %NT/%NX, clicked in a child
--   s7  statuscol    'statuscolumn' with signs, folds, virtual lines
--   s8  others       'rulerformat'/'tabline'/'winbar' over the alphabet
--   s9  errors       malformed formats; the message path in a child
--   s91 crashprobe   the inputs that may kill the editor, one child each
--
-- Every section ends with a `## <name> rows=N` line.  A sweep that goes
-- silently empty otherwise looks exactly like a healthy one.

local uv = vim.uv or vim.loop

local work = assert(os.getenv('STL_WORK'), 'STL_WORK unset')
local runtime = os.getenv('VIMRUNTIME') or ''
local script = debug.getinfo(1, 'S').source:sub(2)

local argv = _G.arg or {}
local child_mode = argv[1]

local only = os.getenv('STLSWEEP_ONLY')
if only == '' then
  only = nil
end
local trace = os.getenv('STLSWEEP_TRACE') == '1'

io.stdout:setvbuf(child_mode and 'no' or 'line')

local rows = 0
local function emit(...)
  rows = rows + 1
  io.write(table.concat({ ... }, ' '), '\n')
end

-- --------------------------------------------------------------- scrub

--- Strip the bits of an answer that name where -- or when -- the run
--- happened.  `%f`/`%F`/`%t` print buffer names, `:help` puts an
--- absolute $VIMRUNTIME path in one, and s9's child quotes the script.
local function scrub(text)
  text = tostring(text)
  text = text:gsub(vim.pesc(work), '<WORK>')
  text = text:gsub(vim.pesc(script), '<SCRIPT>')
  if runtime ~= '' then
    text = text:gsub(vim.pesc(runtime), '<RT>')
  end
  text = text:gsub('nvim%.%d+%.%d+', 'nvim.<PID>.<SEQ>')
  text = text:gsub('nvim%.[%w_.-]+/[%w]+', 'nvim.<U>/<T>')
  -- A long run of one byte is the LENGTH of an s91 input, not the run.
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

local function cap(text, limit)
  limit = limit or 300
  if #text <= limit then
    return text
  end
  return text:sub(1, limit) .. string.format('...<+%d>', #text - limit)
end

--- Escape to one printable line.  Formats carry control characters and
--- the fill characters are multibyte.
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

local structfd
if not child_mode then
  structfd = assert(io.open(assert(os.getenv('STL_STRUCT'), 'STL_STRUCT unset'), 'w'))
end

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

--- Normalise an error to its message.  A pcall against an API function
--- prefixes the Lua source position, which is a line number in THIS
--- file and would re-baseline the whole artifact on any edit above.
local function errtext(res)
  local s = tostring(res)
  s = s:gsub('\nstack traceback:.*$', '')
  s = s:gsub('^[^\n]-stlsweep%.lua:%d+: ', '')
  s = s:gsub('^%[string "[^"]*"%]:%d+: ', '')
  s = s:gsub('\r?\n', ' | ')
  return cap(scrub(s))
end

local function ins(value)
  return (vim.inspect(value, { newline = ' ', indent = '' }))
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
  -- Reset the world.  Sections open windows, tabs and floats, and a
  -- later section's window context is part of every answer it gives.
  pcall(vim.cmd, 'silent! tabonly')
  pcall(vim.cmd, 'silent! only')
  pcall(vim.cmd, 'silent! cclose')
  pcall(vim.cmd, 'silent! helpclose')
  pcall(function()
    vim.o.laststatus = 2
    vim.o.showtabline = 1
    vim.o.ruler = false
    vim.o.showcmd = false
    vim.o.fillchars = ''
    vim.o.statusline = ''
    vim.o.tabline = ''
    vim.o.rulerformat = ''
    vim.o.winbar = ''
    vim.o.statuscolumn = ''
    vim.wo.number = false
    vim.wo.relativenumber = false
    vim.wo.signcolumn = 'auto'
    vim.wo.foldcolumn = '0'
    vim.wo.wrap = true
  end)
  secrows = rows
  emit('##', name)
  local ok, err = pcall(fn)
  if not ok then
    emit('##', name, 'RAISED', esc(scrub(errtext(err))))
  end
  emit('##', name, string.format('rows=%d', rows - secrows - 1))
end

-- --------------------------------------------------------------- eval

--- THE one answer shape.  `w=` is the reported display width, which is
--- the number a truncation bug moves without moving the text, and the
--- text is escaped so a fillchar or a double-width glyph stays on one
--- line.  Every call passes an explicit `maxwidth`: the default is the
--- window's width, and a section that splits a window would otherwise
--- silently change every answer after it.
local WIDE = 400
local function ev(label, fmt, opts)
  label_once(label)
  opts = opts or {}
  if opts.maxwidth == nil then
    opts.maxwidth = WIDE
  end
  local ok, res = pcall(vim.api.nvim_eval_statusline, fmt, opts)
  if ok then
    emit(label, '=', 'w=' .. tostring(res.width), esc(scrub(res.str or '')))
    local rec = { str = res.str, width = res.width }
    if res.highlights then
      local hl = {}
      for i, h in ipairs(res.highlights) do
        hl[i] = { start = h.start, group = h.group, groups = h.groups }
      end
      rec.hl = hl
      emit(label, 'H', esc(scrub(ins(hl))))
    end
    struct(label, rec)
  else
    emit(label, '!', esc(errtext(res)))
    struct(label, { err = errtext(res) })
  end
end

--- An extra question a case asks about the world rather than about a
--- format: buffer numbers, option values, window layout.
local function ask(label, value)
  label_once(label)
  emit(label, 'A', esc(scrub(ins(value))))
  struct(label, { a = ins(value) })
end

-- ------------------------------------------------------------- fixture

local function mkdirp(path, mode)
  local at = ''
  for part in path:gmatch('[^/]+') do
    at = at .. '/' .. part
    uv.fs_mkdir(at, mode or 493)
  end
end

local function writebytes(path, bytes)
  -- Parenthesised: gsub returns (string, count) and the count would be
  -- handed to mkdirp as its MODE (the fssweep trap).
  mkdirp((path:gsub('/[^/]*$', '')))
  local fd = assert(uv.fs_open(path, 'w', 420))
  if #bytes > 0 then
    uv.fs_write(fd, bytes)
  end
  uv.fs_close(fd)
end

local FIXDIR = work .. '/f'
local function fixture()
  mkdirp(FIXDIR)
  writebytes(FIXDIR .. '/named.txt', 'alpha\nbravo\ncharlie\ndelta\necho\n')
  writebytes(FIXDIR .. '/ro.txt', 'read\nonly\nfile\n')
  -- A short-form modeline setting BOTH expression options.  It only
  -- takes effect when 'modelineexpr' is on (otherwise E992), and that
  -- is the only way to make `was_set_insecurely` true, which is the
  -- only way `build_stl_str_hl` ever sees `use_sandbox`.
  writebytes(
    FIXDIR .. '/mode.txt',
    'one\ntwo\n# vim: statusline=%{StlProbe()} rulerformat=%20(%{StlProbe()}%)\n'
  )
  writebytes(FIXDIR .. '/wide.txt', '\228\184\128\228\184\130x\195\169y\n')
  writebytes(FIXDIR .. '/argA.txt', 'A\n')
  writebytes(FIXDIR .. '/argB.txt', 'B\n')
  writebytes(FIXDIR .. '/argC.txt', 'C\n')
end

-- ============================================================ crashprobe
-- Built before the sections so the child dispatch can run without them.

local CRASH = {}
do
  local function add(name, fmt, opts)
    CRASH[#CRASH + 1] = { name, fmt, opts }
  end
  local big = string.rep('x', 8192)
  add('lit-8k', big)
  add('pct-run-4k', string.rep('%', 4096))
  add('pct-pct-4k', string.rep('%%', 4096))
  add('open-group-4k', string.rep('%(', 4096))
  add('balanced-group-2k', string.rep('%(', 2048) .. 'x' .. string.rep('%)', 2048))
  add('close-group-4k', string.rep('%)', 4096))
  add('trunc-run-4k', string.rep('%<', 4096))
  add('sep-run-4k', string.rep('%=', 4096))
  add('sep-run-in-group', '%(' .. string.rep('%=', 2048) .. '%)')
  add('item-run-8k', string.rep('%f', 4096))
  add('hl-run-4k', string.rep('%#Todo#', 4096))
  add('userhl-run-4k', string.rep('%1*', 4096))
  add('comb-run-4k', string.rep('%$Todo$', 4096))
  add('hl-name-8k', '%#' .. big .. '#x')
  add('open-brace-4k', string.rep('%{', 4096))
  add('brace-expr-8k', '%{' .. big .. '}')
  add('reeval-run-2k', string.rep('%{%\'\'%}', 2048))
  add('click-run-2k', string.rep('%@Nope@x%X', 2048))
  add('click-name-8k', '%@' .. big .. '@x%X')
  add('click-unterminated', '%@' .. big)
  add('tab-run-4k', string.rep('%1T', 4096), { use_tabline = true })
  add('tabclose-run-4k', string.rep('%1X', 4096), { use_tabline = true })
  add('width-huge-f', '%2147483647f')
  add('width-overflow-f', '%99999999999999999999f')
  add('width-huge-group', '%2147483647(x%)')
  add('prec-huge-f', '%.2147483647f')
  add('prec-overflow-f', '%.99999999999999999999f')
  add('width-and-prec-huge', '%2147483647.2147483647f')
  add('digits-8k', '%' .. string.rep('9', 8192) .. 'f')
  add('maxwidth-huge', 'abc%<def', { maxwidth = 2147483647 })
  add('maxwidth-neg', 'abc%<def', { maxwidth = -2147483648 })
  add('maxwidth-zero', 'abc%<def', { maxwidth = 0 })
  add('fillchar-wide', 'a%=b', { maxwidth = 40, fillchar = '\228\184\128' })
  add('statuscol-huge-lnum', '%l%s%C', { use_statuscol_lnum = 2147483647 })
  add('statuscol-neg-lnum', '%l%s%C', { use_statuscol_lnum = -2147483648 })
  add('bang-self', '%!g:stl_self')
  add('reeval-self', '%{%g:stl_self%}')
  add('reeval-grow', '%{%g:stl_grow%}', { maxwidth = 100000 })
  add('eval-deep-nest', '%{' .. string.rep('(', 400) .. '1' .. string.rep(')', 400) .. '}')
  add('nested-group-eval', string.rep('%(%{1}', 200) .. string.rep('%)', 200))
  add('wide-glyph-cut', string.rep('\228\184\128', 2000) .. '%<x', { maxwidth = 7 })
  add('combining-cut', ('a\204\129'):rep(2000) .. '%<x', { maxwidth = 7 })
end

local function crashline(i)
  local c = CRASH[i]
  vim.g.stl_self = '%{%g:stl_self%}'
  vim.g.stl_grow = '%{%"a"..g:stl_grow%}'
  local opts = vim.deepcopy(c[3] or {})
  if opts.maxwidth == nil then
    opts.maxwidth = WIDE
  end
  local ok, res = pcall(vim.api.nvim_eval_statusline, c[2], opts)
  io.write(
    i,
    ' k91/',
    c[1],
    ' X ',
    esc(
      scrub(
        ok and ('= w=' .. tostring(res.width) .. ' ' .. tostring(res.str)) or ('! ' .. errtext(res))
      )
    ),
    '\n'
  )
end

if child_mode == '--crash' then
  crashline(assert(tonumber(argv[2]), 'crash index'))
  os.exit(0)
end

-- ==================================================================== s1
-- The 41 item letters x 9 buffer states.
-- ====================================================================

--- Every distinct spelling `build_stl_str_hl`'s item switch can reach.
--- Named, not spelled, in the label: a label that carried `%` would be
--- unreadable in a diff and `%<` would collide with the escaping.
local ITEMS = {
  { 'pct', '[%%]' },
  { 'F-fullpath', '[%F]' },
  { 'f-filepath', '[%f]' },
  { 't-filename', '[%t]' },
  { 'm-modified', '[%m]' },
  { 'M-modifiedalt', '[%M]' },
  { 'r-roflag', '[%r]' },
  { 'R-roflagalt', '[%R]' },
  { 'h-helpflag', '[%h]' },
  { 'H-helpflagalt', '[%H]' },
  { 'w-previewflag', '[%w]' },
  { 'W-previewflagalt', '[%W]' },
  { 'y-filetype', '[%y]' },
  { 'Y-filetypealt', '[%Y]' },
  { 'q-quickfix', '[%q]' },
  { 'n-bufno', '[%n]' },
  { 'b-byteval', '[%b]' },
  { 'B-bytevalx', '[%B]' },
  { 'o-offset', '[%o]' },
  { 'O-offsetx', '[%O]' },
  { 'k-keymap', '[%k]' },
  { 'L-numlines', '[%L]' },
  { 'l-line', '[%l]' },
  { 'c-column', '[%c]' },
  { 'v-virtcol', '[%v]' },
  { 'V-virtcolalt', '[%V]' },
  { 'p-percentage', '[%p]' },
  { 'P-altpercent', '[%P]' },
  { 'a-argliststat', '[%a]' },
  { 'N-pagenum', '[%N]' },
  { 'S-showcmd', '[%S]' },
  { 'C-foldcol', '[%C]' },
  { 's-signcol', '[%s]' },
  { 'eq-separate', '[a%=b]' },
  { 'lt-truncmark', '[abc%<def]' },
  { 'star-userhl0', '[%*x]' },
  { 'star-userhl1', '[%1*x]' },
  { 'star-userhl9', '[%9*x]' },
  { 'hash-highlight', '[%#Todo#x]' },
  { 'hash-unknown', '[%#NoSuchGroup#x]' },
  { 'dollar-comb', '[%$Todo$x]' },
  { 'T-tabpagenr', '[%1Tx]' },
  { 'X-tabclosenr', '[%1Xx]' },
  { 'at-clickfunc', '[%@Nope@x%X]' },
  { 'brace-vimexpr', '[%{1+1}]' },
  { 'brace-reeval', "[%{%'z'%}]" },
  { 'paren-group', '[%(y%)]' },
  { 'paren-empty', '[%(%)]' },
}

--- The nine buffer states.  Each returns a description of what it left
--- current; the caller evaluates the whole alphabet against it.
local STATES = {}

local function state_reset()
  pcall(vim.cmd, 'silent! tabonly')
  pcall(vim.cmd, 'silent! only')
  pcall(vim.cmd, 'silent! cclose')
  pcall(vim.cmd, 'silent! helpclose')
  pcall(vim.cmd, 'silent! enew!')
  vim.wo.previewwindow = false
  vim.o.readonly = false
end

--- Park the cursor somewhere with bytes both sides of it, so `%c`, `%v`,
--- `%b`, `%o` and `%p` are not all reading line 1 column 1.
local function park(lnum, col)
  pcall(vim.api.nvim_win_set_cursor, 0, { lnum, col })
end

STATES[#STATES + 1] = {
  'empty',
  function()
    state_reset()
    vim.bo.buftype = 'nofile'
    vim.bo.bufhidden = 'hide'
    vim.bo.swapfile = false
  end,
}
STATES[#STATES + 1] = {
  'named',
  function()
    state_reset()
    vim.cmd('silent! edit ' .. vim.fn.fnameescape(FIXDIR .. '/named.txt'))
    park(3, 2)
  end,
}
STATES[#STATES + 1] = {
  'modified',
  function()
    state_reset()
    vim.cmd('silent! edit ' .. vim.fn.fnameescape(FIXDIR .. '/named.txt'))
    vim.api.nvim_buf_set_lines(0, 1, 2, false, { 'BRAVO-CHANGED' })
    park(2, 3)
  end,
}
STATES[#STATES + 1] = {
  'readonly',
  function()
    state_reset()
    vim.cmd('silent! edit ' .. vim.fn.fnameescape(FIXDIR .. '/ro.txt'))
    vim.bo.readonly = true
    vim.bo.modifiable = false
    park(2, 1)
  end,
}
STATES[#STATES + 1] = {
  'help',
  function()
    state_reset()
    pcall(vim.cmd, 'silent! help')
    park(4, 0)
  end,
}
STATES[#STATES + 1] = {
  'preview',
  function()
    state_reset()
    vim.cmd('silent! edit ' .. vim.fn.fnameescape(FIXDIR .. '/named.txt'))
    vim.wo.previewwindow = true
    park(2, 0)
  end,
}
STATES[#STATES + 1] = {
  'quickfix',
  function()
    state_reset()
    vim.fn.setqflist({
      { filename = FIXDIR .. '/named.txt', lnum = 2, text = 'quick' },
      { filename = FIXDIR .. '/ro.txt', lnum = 1, text = 'fix' },
    })
    pcall(vim.cmd, 'silent! copen')
    park(2, 0)
  end,
}
STATES[#STATES + 1] = {
  'terminal',
  function()
    state_reset()
    -- `nvim_open_term` gives a REAL `buftype=terminal` buffer with NO
    -- job behind it.  `:terminal` would spawn a shell whose output --
    -- and whose `term://<cwd>//<pid>:<cmd>` name -- are not
    -- reproducible, and `set buftype=terminal` is rejected outright
    -- (E474).  The buffer is sized by the window, so 'lines' is pinned.
    local buf = vim.api.nvim_create_buf(false, true)
    vim.api.nvim_set_current_buf(buf)
    local chan = vim.api.nvim_open_term(buf, {})
    vim.api.nvim_chan_send(chan, 'terminal-one\r\nterminal-two\r\n')
    vim.wait(2000, function()
      return (vim.api.nvim_buf_get_lines(0, 1, 2, false)[1] or '') == 'terminal-two'
    end, 10)
    park(2, 4)
  end,
}
STATES[#STATES + 1] = {
  'unnamed',
  function()
    state_reset()
    vim.api.nvim_buf_set_lines(0, 0, -1, false, { 'no name here', 'second line' })
    vim.bo.modified = false
    park(1, 5)
  end,
}

section('s1-items', function()
  fixture()
  for _, st in ipairs(STATES) do
    local name, setup = st[1], st[2]
    setup()
    ask(
      's1/' .. name .. '#ctx',
      {
        bt = vim.bo.buftype,
        ft = vim.bo.filetype,
        mod = vim.bo.modified,
        ro = vim.bo.readonly,
        ma = vim.bo.modifiable,
        lines = vim.api.nvim_buf_line_count(0),
        cursor = vim.api.nvim_win_get_cursor(0),
        prev = vim.wo.previewwindow,
        name = vim.api.nvim_buf_get_name(0),
      }
    )
    for _, it in ipairs(ITEMS) do
      ev('s1/' .. name .. '/' .. it[1], it[2])
    end
    -- The same alphabet once more with the highlight records on, which
    -- is the only view of the `stl_hlrec_t` half of the answer.
    ev('s1/' .. name .. '#hl', '%1*a%#Todo#b%$Comment$c%*d%(e%)', { highlights = true })
    ev('s1/' .. name .. '#default', '%<%f %h%m%r%=%-14.14(%l,%c%V%) %P', { highlights = true })
  end
  state_reset()
end)

-- ==================================================================== s2
-- Minimum width, maximum width, precision and the numeric argument
-- forms.  `minwid` is clamped at 50 and `maxwid` defaults to 9999.
-- ====================================================================

section('s2-width', function()
  fixture()
  vim.cmd('silent! edit ' .. vim.fn.fnameescape(FIXDIR .. '/named.txt'))
  park(3, 2)

  local WIDTHS = {
    { 'w0', '0' },
    { 'w1', '1' },
    { 'w5', '5' },
    { 'w49', '49' },
    { 'w50', '50' },
    { 'w51', '51' },
    { 'w100', '100' },
    { 'wneg5', '-5' },
    { 'wneg0', '-0' },
    { 'wneg50', '-50' },
    { 'wneg51', '-51' },
    { 'wlead05', '05' },
    { 'whuge', '2000000000' },
    { 'wmax', '2147483647' },
    { 'wover', '99999999999999999999' },
  }
  local TARGETS = {
    { 'f', 'f' },
    { 'l', 'l' },
    { 'L', 'L' },
    { 'expr', "{'abcdefghij'}" },
  }
  for _, w in ipairs(WIDTHS) do
    for _, t in ipairs(TARGETS) do
      ev('s2/min/' .. w[1] .. '/' .. t[1], '[%' .. w[2] .. t[2] .. ']')
    end
  end

  local PRECS = {
    { 'p0', '.0' },
    { 'p1', '.1' },
    { 'p3', '.3' },
    { 'p10', '.10' },
    { 'p50', '.50' },
    { 'phuge', '.2000000000' },
    { 'pover', '.99999999999999999999' },
    { 'pempty', '.' },
  }
  for _, p in ipairs(PRECS) do
    ev('s2/max/' .. p[1] .. '/f', '[%' .. p[2] .. 'f]')
    ev('s2/max/' .. p[1] .. '/expr', "[%" .. p[2] .. "{'abcdefghij'}]")
    ev('s2/max/' .. p[1] .. '/group', '[%' .. p[2] .. '(abcdefghij%)]')
  end

  local BOTH = {
    { 'b20.10', '20.10' },
    { 'b-20.10', '-20.10' },
    { 'b5.3', '5.3' },
    { 'b-5.3', '-5.3' },
    { 'b3.5', '3.5' },
    { 'b0.0', '0.0' },
    { 'b50.1', '50.1' },
    { 'b60.60', '60.60' },
  }
  for _, b in ipairs(BOTH) do
    ev('s2/both/' .. b[1] .. '/f', '[%' .. b[2] .. 'f]')
    ev('s2/both/' .. b[1] .. '/F', '[%' .. b[2] .. 'F]')
    ev('s2/both/' .. b[1] .. '/group', '[%' .. b[2] .. '(abcdefghij%)]')
  end

  -- The numeric argument on the items that take one for a reason other
  -- than width: user highlight, tab page number, tab close, click.
  for _, n in ipairs({ '', '0', '1', '5', '9', '10', '49', '50', '51', '100', '2147483647' }) do
    local key = n == '' and 'none' or n
    ev('s2/arg/star/' .. key, '[%' .. n .. '*x]')
    ev('s2/arg/T/' .. key, '[%' .. n .. 'Tx]', { use_tabline = true })
    ev('s2/arg/X/' .. key, '[%' .. n .. 'Xx]', { use_tabline = true })
    ev('s2/arg/click/' .. key, '[%' .. n .. '@Nope@x%X]')
  end
  for _, n in ipairs({ '-1', '-9', '-50' }) do
    ev('s2/arg/star/neg' .. n:sub(2), '[%' .. n .. '*x]')
    ev('s2/arg/click/neg' .. n:sub(2), '[%' .. n .. '@Nope@x%X]')
  end

  -- Width interacting with the harness's own maxwidth.
  for _, mw in ipairs({ 1, 2, 5, 10, 20, 60 }) do
    ev('s2/maxwidth/' .. mw .. '/min20', '%20f', { maxwidth = mw })
    ev('s2/maxwidth/' .. mw .. '/group20', '%20(ab%)', { maxwidth = mw })
    ev('s2/maxwidth/' .. mw .. '/sep', 'a%=b', { maxwidth = mw })
  end
  ask('s2#opts', { columns = vim.o.columns, lines = vim.o.lines, wide = WIDE })
end)

-- ==================================================================== s3
-- Groups: nesting, the empty-group elision rule, `%=` inside and
-- outside, several `%=`, and a group's width against `%<`.
-- ====================================================================

section('s3-groups', function()
  fixture()
  vim.cmd('silent! edit ' .. vim.fn.fnameescape(FIXDIR .. '/named.txt'))
  park(3, 2)

  local NEST = {
    { 'flat', '%(a%)' },
    { 'two', '%(a%(b%)c%)' },
    { 'three', '%(a%(b%(c%)d%)e%)' },
    { 'four', '%(%(%(%(x%)%)%)%)' },
    { 'ten', string.rep('%(', 10) .. 'x' .. string.rep('%)', 10) },
    { 'sixty', string.rep('%(', 60) .. 'x' .. string.rep('%)', 60) },
    { 'adjacent', '%(a%)%(b%)%(c%)' },
    { 'width-outer', '%20(a%(b%)c%)' },
    { 'width-inner', '%(a%10(b%)c%)' },
    { 'width-both', '%20(a%10(b%)c%)' },
    { 'prec-outer', '%.4(abcdefgh%)' },
    { 'prec-inner', '%(ab%.2(cdef%)gh%)' },
  }
  for _, n in ipairs(NEST) do
    ev('s3/nest/' .. n[1], '[' .. n[2] .. ']')
  end

  -- Elision: a group whose items all produced nothing is dropped whole,
  -- literal text inside it included.  This is the single most
  -- surprising rule in the item loop and the one a rewrite loses first.
  local ELIDE = {
    { 'empty', '%(%)' },
    { 'close-in-group', '%()%)' },
    { 'literal-only', '%(lit%)' },
    { 'expr-empty', "%(%{''}%)" },
    { 'expr-empty-lit', "%(x%{''}y%)" },
    { 'expr-nonempty-lit', "%(x%{'v'}y%)" },
    { 'two-empty-exprs', "%(%{''}%{''}%)" },
    { 'one-of-two', "%(%{''}%{'v'}%)" },
    { 'space-only', '%( %)' },
    { 'hl-only', '%(%1*%)' },
    { 'hl-and-empty-expr', "%(%1*%{''}%)" },
    { 'sep-only', '%(%=%)' },
    { 'trunc-only', '%(%<%)' },
    { 'flag-empty', '%(%m%)' },
    { 'flag-empty-lit', '%([%m]%)' },
    { 'nested-empty', "%(a%(%{''}%)b%)" },
    { 'nested-empty-only', "%(%(%{''}%)%)" },
    { 'width-on-empty', "%10(%{''}%)" },
    { 'widthneg-on-empty', "%-10(%{''}%)" },
  }
  for _, e in ipairs(ELIDE) do
    ev('s3/elide/' .. e[1], '<' .. e[2] .. '>')
  end

  local SEP = {
    { 'one', 'a%=b' },
    { 'two', 'a%=b%=c' },
    { 'three', 'a%=b%=c%=d' },
    { 'four', 'a%=b%=c%=d%=e' },
    { 'leading', '%=tail' },
    { 'trailing', 'head%=' },
    { 'only', '%=' },
    { 'in-group', '%(a%=b%)' },
    { 'in-group-outer', 'x%(a%=b%)y%=z' },
    { 'group-then-sep', '%(ab%)%=%(cd%)' },
    { 'sep-in-nested', '%(a%(b%=c%)d%)' },
    { 'sep-with-width', 'a%=%20fb' },
  }
  for _, mw in ipairs({ 10, 20, 40 }) do
    for _, s in ipairs(SEP) do
      ev('s3/sep/' .. s[1] .. '/mw' .. mw, s[2], { maxwidth = mw })
    end
  end
  for _, s in ipairs(SEP) do
    ev('s3/sepfill/' .. s[1], s[2], { maxwidth = 24, fillchar = '.' })
  end

  -- Group width against the truncation mark.
  for _, mw in ipairs({ 6, 12, 30 }) do
    ev('s3/gt/wide-group/mw' .. mw, '%30(abcdefghij%)', { maxwidth = mw })
    ev('s3/gt/group-trunc/mw' .. mw, '%(abc%<defghij%)', { maxwidth = mw })
    ev('s3/gt/trunc-before/mw' .. mw, 'abc%<%(defghij%)', { maxwidth = mw })
    ev('s3/gt/trunc-after/mw' .. mw, '%(abcdef%)%<ghij', { maxwidth = mw })
    ev('s3/gt/sep-and-trunc/mw' .. mw, 'a%=b%<cdefghij', { maxwidth = mw })
  end

  ev('s3/hl/group-userhl', '%1*%(a%2*b%)c', { highlights = true })
  ev('s3/hl/group-elided-userhl', "%1*%(%{''}%2*%)c", { highlights = true })
  ev('s3/hl/nested-restore', '%1*a%(%2*b%)c', { highlights = true })
end)

-- ==================================================================== s4
-- Truncation.  `%<` at every position of one fixed format, maxwidth
-- below the content, `%<` inside a group, and the multibyte and
-- double-width cases at the cut point.
-- ====================================================================

section('s4-trunc', function()
  fixture()
  vim.cmd('silent! edit ' .. vim.fn.fnameescape(FIXDIR .. '/named.txt'))
  park(3, 2)

  -- `%<` walked across a fixed twenty-character body, one position at a
  -- time, at three maxwidths.  This is the shape of the cut, and a
  -- one-byte error in `%<`'s handling moves exactly one of these rows.
  local BODY = 'abcdefghijklmnopqrst'
  for pos = 0, #BODY do
    local fmt = BODY:sub(1, pos) .. '%<' .. BODY:sub(pos + 1)
    for _, mw in ipairs({ 5, 12, 25 }) do
      ev(string.format('s4/pos/%02d/mw%d', pos, mw), fmt, { maxwidth = mw })
    end
  end

  -- No `%<` at all: the cut is the implicit one at the end.
  for _, mw in ipairs({ 0, 1, 2, 3, 5, 19, 20, 21 }) do
    ev('s4/nomark/mw' .. mw, BODY, { maxwidth = mw })
  end

  -- Two marks, and marks inside groups.
  local MULTI = {
    { 'two-marks', 'aaa%<bbb%<ccc' },
    { 'three-marks', 'a%<b%<c%<d' },
    { 'mark-in-group', '%(aaa%<bbb%)ccc' },
    { 'mark-outside-group', 'aaa%<%(bbbccc%)' },
    { 'mark-in-nested', '%(a%(b%<c%)d%)efgh' },
    { 'mark-then-sep', 'aaa%<bbb%=ccc' },
    { 'sep-then-mark', 'aaa%=bbb%<ccc' },
    { 'mark-first', '%<abcdefghijklmnop' },
    { 'mark-last', 'abcdefghijklmnop%<' },
    { 'mark-with-item', '%f%<%l,%c' },
    { 'mark-in-width', '%20(ab%<cd%)' },
  }
  for _, m in ipairs(MULTI) do
    for _, mw in ipairs({ 4, 8, 16 }) do
      ev('s4/multi/' .. m[1] .. '/mw' .. mw, m[2], { maxwidth = mw })
    end
  end

  -- Multibyte and double-width AT the cut.  Each of these strings is
  -- ten display cells wide; the cut lands mid-character at some of the
  -- maxwidths and the answer is whether the cell is dropped or padded.
  local TEXTS = {
    { 'ascii', 'abcdefghij' },
    { 'latin1', '\195\169\195\168\195\170\195\171\195\172\195\173\195\174\195\175\195\176\195\177' },
    { 'cjk', '\228\184\128\228\184\130\228\184\137\229\155\155\228\186\148' },
    { 'cjk-mixed', 'a\228\184\128b\228\184\130c\228\184\137d' },
    { 'combining', 'a\204\129b\204\129c\204\129d\204\129e\204\129f\204\129g\204\129h\204\129i\204\129j\204\129' },
    { 'emoji', '\240\159\152\128\240\159\152\129\240\159\152\130\240\159\152\131\240\159\152\132' },
    { 'tab', 'ab\tcd\tef\tgh' },
  }
  for _, t in ipairs(TEXTS) do
    for mw = 1, 12 do
      ev('s4/wide/' .. t[1] .. '/mw' .. mw, t[2], { maxwidth = mw })
      ev('s4/widemark/' .. t[1] .. '/mw' .. mw, '%<' .. t[2], { maxwidth = mw })
      ev('s4/widetail/' .. t[1] .. '/mw' .. mw, t[2] .. '%<' .. t[2], { maxwidth = mw })
    end
  end

  -- The fill character is also measured in cells.  Labelled by INDEX,
  -- not by the character: a fillchar in a label is scrubbed and escaped
  -- along with everything else, and `' '` and `'.'` are the same length.
  local FILLS = { ' ', '.', '\194\183', '\228\184\128', '\240\159\152\128', 'ab' }
  for fi, fc in ipairs(FILLS) do
    for _, mw in ipairs({ 6, 11, 20 }) do
      ev(string.format('s4/fill/%02d/mw%d', fi, mw), 'a%=b', { maxwidth = mw, fillchar = fc })
      ev(string.format('s4/fillg/%02d/mw%d', fi, mw), '%20(x%)', { maxwidth = mw, fillchar = fc })
    end
  end
end)

-- ==================================================================== s5
-- Evaluation: `%{}`, `%{%..%}`, `%!`, errors, the sandbox and the
-- MAX_STL_EVAL_DEPTH == 100 wall.
-- ====================================================================

section('s5-eval', function()
  fixture()
  vim.cmd('silent! edit ' .. vim.fn.fnameescape(FIXDIR .. '/named.txt'))
  park(3, 2)
  vim.cmd([[
    func! StlNum() abort
      return 42
    endfunc
    func! StlStr() abort
      return 'from-func'
    endfunc
    func! StlList() abort
      return [1, 2]
    endfunc
    func! StlDict() abort
      return {'a': 1}
    endfunc
    func! StlFloat() abort
      return 1.5
    endfunc
    func! StlNull() abort
      return v:null
    endfunc
    func! StlThrow() abort
      throw 'stl-thrown'
    endfunc
    func! StlErr() abort
      return nosuchvariable
    endfunc
    func! StlSideEffect() abort
      let g:stl_side += 1
      return g:stl_side
    endfunc
    func! StlPercent() abort
      return '100%'
    endfunc
    func! StlFmt() abort
      return '%l,%c'
    endfunc
    func! StlSystem() abort
      return system('echo hi')
    endfunc
  ]])
  vim.g.stl_side = 0

  local EXPRS = {
    { 'num', '1+1' },
    { 'str', "'plain'" },
    { 'empty-str', "''" },
    { 'func-num', 'StlNum()' },
    { 'func-str', 'StlStr()' },
    { 'func-list', 'StlList()' },
    { 'func-dict', 'StlDict()' },
    { 'func-float', 'StlFloat()' },
    { 'func-null', 'StlNull()' },
    { 'func-throw', 'StlThrow()' },
    { 'func-err', 'StlErr()' },
    { 'undefined-var', 'g:no_such_variable_at_all' },
    { 'undefined-func', 'NoSuchFunctionHere()' },
    { 'percent-in-result', 'StlPercent()' },
    { 'format-in-result', 'StlFmt()' },
    { 'newline-in-result', [["a\nb"]] },
    { 'nul-in-result', [[nr2char(10)]] },
    { 'multibyte', [["一丂"]] },
    { 'long', "repeat('z', 300)" },
    { 'winnr', 'winnr()' },
    { 'bufnr', 'bufnr()' },
    { 'lnum', 'line(".")' },
    { 'vlua', 'luaeval("1+2")' },
    { 'nested-braces', "get({'a': 'b'}, 'a')" },
    { 'has-key-brace', "has_key({'x':1}, 'x')" },
  }
  for _, e in ipairs(EXPRS) do
    ev('s5/brace/' .. e[1], '[%{' .. e[2] .. '}]')
    ev('s5/brace-w/' .. e[1], '[%-12.6{' .. e[2] .. '}]')
  end

  -- `%{%..%}`: the result is re-parsed as a format.
  local REEVAL = {
    { 'plain', "'x'" },
    { 'item', "'%l,%c'" },
    { 'group', "'%(a%)'" },
    { 'sep', "'a%=b'" },
    { 'trunc', "'abc%<def'" },
    { 'nested-brace', [["%{1+1}"]] },
    { 'nested-reeval', [["%{%'q'%}"]] },
    { 'click', "'%@Nope@x%X'" },
    { 'hl', "'%1*x'" },
    { 'bad', "'%Z'" },
    { 'unbalanced', "'%('" },
    { 'empty', "''" },
    { 'percent', "'100%%'" },
    { 'bare-percent', "'100%'" },
  }
  for _, r in ipairs(REEVAL) do
    ev('s5/reeval/' .. r[1], '[%{%' .. r[2] .. '%}]')
  end

  -- `%!` -- only recognised at offset 0 of the whole format.
  local BANG = {
    { 'str', "%!'from-bang'" },
    { 'item', [[%!'%l,%c']] },
    { 'func', '%!StlStr()' },
    { 'num', '%!1+1' },
    { 'err', '%!NoSuchFunctionHere()' },
    { 'throw', '%!StlThrow()' },
    { 'empty', "%!''" },
    { 'brace', [[%!'%{1+1}']] },
    { 'reeval', [[%!'%{%"r"%}']] },
    { 'bang-again', [[%!'%!1']] },
    { 'not-at-start', 'x%!1' },
    { 'bang-alone', '%!' },
  }
  for _, b in ipairs(BANG) do
    ev('s5/bang/' .. b[1], b[2])
  end

  -- How many times does one `%{}` run in one build?  A rewrite that
  -- evaluates twice (measure, then emit) is invisible to every other
  -- row here.
  vim.g.stl_side = 0
  ev('s5/side/once', '[%{StlSideEffect()}]')
  ask('s5/side/once#count', vim.g.stl_side)
  vim.g.stl_side = 0
  ev('s5/side/in-group', '[%(%{StlSideEffect()}%)]')
  ask('s5/side/in-group#count', vim.g.stl_side)
  vim.g.stl_side = 0
  ev('s5/side/elided', "[%(%{StlSideEffect()}%{''}%)]")
  ask('s5/side/elided#count', vim.g.stl_side)
  vim.g.stl_side = 0
  ev('s5/side/truncated', '[%{StlSideEffect()}]', { maxwidth = 1 })
  ask('s5/side/truncated#count', vim.g.stl_side)

  -- THE DEPTH WALL.  MAX_STL_EVAL_DEPTH is 100: a `%{%..%}` whose
  -- result is another `%{%..%}` is re-parsed until the counter runs
  -- out, and then the text is emitted literally.  Counting the a's is
  -- the only direct measurement of the constant there is.
  vim.g.stl_grow = '%{%"a"..g:stl_grow%}'
  vim.g.stl_self = '%{%g:stl_self%}'
  vim.g.stl_pair = '%{%"b".g:stl_pair."c"%}'
  ev('s5/depth/grow', '%{%g:stl_grow%}', { maxwidth = 100000 })
  ev('s5/depth/self', '%{%g:stl_self%}', { maxwidth = 100000 })
  ev('s5/depth/pair', '%{%g:stl_pair%}', { maxwidth = 100000 })
  ev('s5/depth/two-at-once', '%{%g:stl_grow%}%{%g:stl_grow%}', { maxwidth = 100000 })
  ev('s5/depth/in-group', '%(%{%g:stl_grow%}%)', { maxwidth = 100000 })
  ev('s5/depth/bang-grow', '%!g:stl_grow', { maxwidth = 100000 })
  do
    local ok, res = pcall(vim.api.nvim_eval_statusline, '%{%g:stl_grow%}', { maxwidth = 100000 })
    ask('s5/depth/grow#count', ok and #(res.str or '') or ('ERR ' .. errtext(res)))
  end

  -- Window and buffer context: which window does the expression see?
  vim.cmd('silent! vsplit')
  local other = vim.api.nvim_get_current_win()
  vim.cmd('silent! wincmd p')
  ev('s5/ctx/here', '[%{winnr()}|%{bufnr()}|%{line(".")}]')
  ev('s5/ctx/other-win', '[%{winnr()}|%{bufnr()}|%{line(".")}]', { winid = other })
  ev('s5/ctx/other-win-f', '[%f|%l|%c]', { winid = other })
  ask('s5/ctx#wins', { cur = vim.api.nvim_get_current_win(), other = other })
  vim.cmd('silent! only')

  -- THE SANDBOX.  `nvim_eval_statusline` hands `build_stl_str_hl`
  -- `kOptInvalid`, so `use_sandbox` is false there no matter what; the
  -- arm is only reachable through a *redraw* of an option that
  -- `was_set_insecurely` says came from a modeline -- and a modeline
  -- may only carry an expression option at all when 'modelineexpr' is
  -- on (without it the answer is E992 and the option is never set, so
  -- this whole block measured nothing on the first draft).  Run in a
  -- child, because both halves of the answer are messages.
  --
  -- `StlProbe` reports which operations the sandbox refuses, by error
  -- number, so the row distinguishes "the sandbox flag was not passed"
  -- from "the expression failed for some other reason".  The control is
  -- the SAME format set from `--cmd`, which is secure.
  -- Written to a FILE and sourced, not passed as a multi-line `--cmd`:
  -- a `\`-continuation inside a `--cmd` argument is not joined, and the
  -- whole block failed with the message `line 2:` sitting on the screen
  -- row this section reports -- which looked exactly like a sandbox
  -- refusal and was not one.
  writebytes(
    work .. '/probe.vim',
    table.concat({
      'func! StlProbe() abort',
      '  let r = []',
      "  let probes = [['sys', \"system('true')\"], ['exe', \"execute('let g:zz=1')\"], ['lua', 'luaeval(\"1\")'], ['wf', \"writefile([], 'sbxprobe')\"], ['del', \"delete('sbxprobe')\"], ['cwd', 'getcwd()'], ['buf', 'bufnr()'], ['fnm', \"fnamemodify('a', ':p')\"]]",
      '  for p in probes',
      '    try',
      '      call eval(p[1])',
      "      call add(r, p[0] . '=ok')",
      '    catch',
      "      call add(r, p[0] . '=' . matchstr(v:exception, 'E\\d\\+'))",
      '    endtry',
      '  endfor',
      "  return join(r, ',')",
      'endfunc',
      '',
    }, '\n')
  )
  local function sandbox_child(name, args)
    local out = work .. '/sandbox-' .. name .. '.txt'
    os.remove(out)
    local argv = { work .. '/bin/nvim', '--headless', '-u', 'NONE', '-i', 'NONE' }
    for _, a in ipairs(args) do
      argv[#argv + 1] = a
    end
    for _, a in ipairs({
      '-c',
      'redraw!',
      '-c',
      "call writefile([&l:statusline, &rulerformat, join(map(range(1, &columns), 'screenstring(&lines - 1, v:val)'), ''), join(map(range(1, &columns), 'screenstring(&lines, v:val)'), '')], '"
        .. out
        .. "')",
      '-c',
      'qa!',
    }) do
      argv[#argv + 1] = a
    end
    local res = vim
      .system(argv, {
        text = true,
        cwd = work,
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
        clear_env = true,
        timeout = 60000,
      })
      :wait()
    local got = {}
    local fh = io.open(out, 'r')
    if fh then
      for line in fh:lines() do
        got[#got + 1] = (line:gsub('%s+$', ''))
      end
      fh:close()
    end
    label_once('s5/sandbox/' .. name)
    emit(
      's5/sandbox/' .. name,
      '=',
      'code=' .. tostring(res.code),
      esc(scrub(table.concat(got, ' // ')))
    )
    struct('s5/sandbox/' .. name, { code = res.code, signal = res.signal, lines = got })
  end
  sandbox_child('modeline', {
    '--cmd',
    'set modeline modelineexpr modelines=5 laststatus=2',
    '-S',
    work .. '/probe.vim',
    '-c',
    'edit ' .. FIXDIR .. '/mode.txt',
  })
  sandbox_child('modeline-off', {
    '--cmd',
    'set modeline nomodelineexpr modelines=5 laststatus=2',
    '-S',
    work .. '/probe.vim',
    '-c',
    'edit ' .. FIXDIR .. '/mode.txt',
  })
  sandbox_child('secure', {
    '--cmd',
    'set laststatus=2',
    '-S',
    work .. '/probe.vim',
    '--cmd',
    'set statusline=%{StlProbe()}',
    '-c',
    'edit ' .. FIXDIR .. '/mode.txt',
  })
  sandbox_child('modeline-ruler', {
    '--cmd',
    'set modeline modelineexpr modelines=5 laststatus=0 ruler',
    '-S',
    work .. '/probe.vim',
    '-c',
    'edit ' .. FIXDIR .. '/mode.txt',
  })
end)

-- ==================================================================== s6
-- Click definitions.  `nvim_eval_statusline` parses `%@Func@..%X` and
-- discards the records, so they are read back by CLICKING -- which
-- needs the main input loop, i.e. an `--embed` child over RPC.
-- ====================================================================

local function embed_start()
  local chan = vim.fn.jobstart({
    work .. '/bin/nvim',
    '--headless',
    '--embed',
    '-u',
    'NONE',
    '-i',
    'NONE',
    '--cmd',
    'set noswapfile',
  }, {
    rpc = true,
    cwd = work,
    clear_env = true,
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
  return chan
end

--- `jobstop` is not enough on its own: the child has to be reaped, or
--- it survives this process and holds $WORK as its cwd while the next
--- run's `rm -rf` races it (the cmdsweep s18 lesson).
local function embed_stop(chan)
  pcall(vim.fn.jobstop, chan)
  pcall(vim.fn.jobwait, { chan }, 10000)
end

section('s6-click', function()
  fixture()
  local chan = embed_start()
  local function lua(code)
    local ok, res = pcall(vim.rpcrequest, chan, 'nvim_exec_lua', code, {})
    if ok then
      return res
    end
    return 'RPCERR ' .. errtext(res)
  end

  lua([[
    vim.o.laststatus = 2
    vim.o.showtabline = 2
    vim.g.stl_clicks = {}
    vim.cmd([==[
      func! StlClick(minwid, clicks, button, mods) abort
        call add(g:stl_clicks, printf('%d/%d/%s/[%s]', a:minwid, a:clicks, a:button, a:mods))
      endfunc
      func! StlClick2(minwid, clicks, button, mods) abort
        call add(g:stl_clicks, printf('two:%d/%d/%s/[%s]', a:minwid, a:clicks, a:button, a:mods))
      endfunc
    ]==])
    vim.api.nvim_buf_set_lines(0, 0, -1, false, {'one','two','three','four','five'})
  ]])

  --- One click scenario: set the format, redraw, click at (row, col),
  --- and report both what the handler recorded and what the screen row
  --- actually says.  `mousetime` is pinned per scenario -- multi-click
  --- detection reads the wall clock, and an unpinned one is the classic
  --- nondeterministic mouse oracle.
  ---
  --- A click entry is `{button, action, modifiers, row, col}`; the grid
  --- argument is always 0.
  local function click(label, setup, clicks, opts)
    opts = opts or {}
    local rowlist = {}
    for i, r in ipairs(opts.rows or { 1 }) do
      rowlist[i] = tostring(r)
    end
    -- Reset every custom line first.  Without this a scenario inherits
    -- the previous one's `'tabline'`, whose click regions sit on screen
    -- row 1 and shadow the winbar's -- the first draft's whole
    -- `s6/winbar/*` block recorded the TABLINE's handler.
    local code = table.concat({
      'vim.g.stl_clicks = {}',
      'vim.cmd("silent! only")',
      'vim.o.laststatus = 2',
      'vim.o.showtabline = 2',
      'vim.o.tabline = ""',
      'vim.o.winbar = ""',
      'vim.o.statusline = ""',
      'vim.o.mousetime = ' .. tostring(opts.mousetime or 0),
      'vim.o.mousemodel = ' .. string.format('%q', opts.mousemodel or 'extend'),
      setup,
      'vim.cmd("mode")',
      'vim.cmd("redraw")',
    }, '\n')
    lua(code)
    for _, c in ipairs(clicks) do
      pcall(vim.rpcrequest, chan, 'nvim_input_mouse', c[1], c[2], c[3] or '', 0, c[4], c[5])
    end
    -- `{...}` and not `vim.json.encode`: a JSON array is `[1,2]`, which
    -- is a syntax error where the generated Lua wants a table
    -- constructor -- the first draft's whole s6 read RPCERR.
    local got = lua([[
      vim.cmd('redraw')
      local rows = {}
      for _, r in ipairs({]] .. table.concat(rowlist, ',') .. [[}) do
        local s = {}
        for c = 1, vim.o.columns do s[#s+1] = vim.fn.screenstring(r, c) end
        rows[#rows+1] = r .. ':' .. (table.concat(s):gsub('%s+$', ''))
      end
      return { clicks = vim.g.stl_clicks, rows = rows,
               mouse = vim.fn.getmousepos() }
    ]])
    label_once(label)
    emit(label, '=', esc(scrub(ins(got))))
    struct(label, got)
  end

  local TABLINE = '%1T%@StlClick@one%X%2T%@StlClick2@two%X%3Tthree%X%999Xclose'
  local STATUS = 'left%0@StlClick@CLICKY%X mid %5@StlClick2@OTHER%X right'
  local WINBAR = 'wb%7@StlClick@BAR%X end'

  -- Where the regions are, before any clicking: the screen row is the
  -- map from column to click definition.
  click('s6/tabline/map', 'vim.o.tabline = ' .. string.format('%q', TABLINE), {}, { rows = { 1 } })
  click(
    's6/status/map',
    'vim.o.statusline = ' .. string.format('%q', STATUS),
    {},
    { rows = { 23 } }
  )

  -- Every button, on the tabline's first region.
  for _, b in ipairs({ 'left', 'right', 'middle', 'x1', 'x2' }) do
    click(
      's6/button/' .. b,
      'vim.o.tabline = ' .. string.format('%q', TABLINE),
      { { b, 'press', '', 0, 1 }, { b, 'release', '', 0, 1 } },
      { rows = { 1 } }
    )
  end

  -- Modifiers.
  for _, m in ipairs({ '', 'S', 'C', 'A', 'M', 'SC', 'SCA' }) do
    click(
      's6/mods/' .. (m == '' and 'none' or m),
      'vim.o.tabline = ' .. string.format('%q', TABLINE),
      { { 'left', 'press', m, 0, 1 }, { 'left', 'release', m, 0, 1 } },
      { rows = { 1 } }
    )
  end

  -- Multi-click.  `mousetime=100000` makes every click a continuation,
  -- `mousetime=0` makes none of them one; both are pinned, and the
  -- difference between the two IS the multi-click counter.
  for _, n in ipairs({ 2, 3, 4, 5 }) do
    local seq = {}
    for _ = 1, n do
      seq[#seq + 1] = { 'left', 'press', '', 0, 1 }
      seq[#seq + 1] = { 'left', 'release', '', 0, 1 }
    end
    click(
      's6/multi/always/' .. n,
      'vim.o.tabline = ' .. string.format('%q', TABLINE),
      seq,
      { rows = { 1 }, mousetime = 100000 }
    )
    click(
      's6/multi/never/' .. n,
      'vim.o.tabline = ' .. string.format('%q', TABLINE),
      seq,
      { rows = { 1 }, mousetime = 0 }
    )
  end

  -- Every column of the tabline, so the region boundaries are exact.
  for col = 0, 17 do
    click(
      string.format('s6/col/%02d', col),
      'vim.o.tabline = ' .. string.format('%q', TABLINE),
      { { 'left', 'press', '', 0, col }, { 'left', 'release', '', 0, col } },
      { rows = { 1 } }
    )
  end

  -- The statusline and the winbar carry their own arenas.
  -- The status line is screen row 23 of 24, i.e. MOUSE row 22: with
  -- `showtabline=2` row 0 is the tabline and a status-line scenario
  -- clicked there records nothing at all.
  for _, col in ipairs({ 0, 4, 5, 10, 11, 15, 16, 21, 22, 30 }) do
    click(
      's6/status/col' .. col,
      'vim.o.statusline = ' .. string.format('%q', STATUS),
      { { 'left', 'press', '', 22, col }, { 'left', 'release', '', 22, col } },
      { rows = { 23 } }
    )
  end
  -- The winbar is BELOW the tabline: with `showtabline=2` it is screen
  -- row 2, i.e. mouse row 1.  Clicking row 0 hits the tabline.
  for _, col in ipairs({ 0, 2, 3, 5, 6, 9 }) do
    click(
      's6/winbar/col' .. col,
      'vim.o.winbar = ' .. string.format('%q', WINBAR),
      { { 'left', 'press', '', 1, col }, { 'left', 'release', '', 1, col } },
      { rows = { 2 } }
    )
  end

  -- `%NT` / `%NX` on a real tabline with three tab pages: the numbers
  -- are tab page ids, and `%X` closes.
  click(
    's6/tabs/select',
    [[
      vim.cmd('silent! tabonly')
      vim.cmd('silent! tabnew')
      vim.cmd('silent! tabnew')
      vim.cmd('silent! tabfirst')
      vim.o.tabline = '%1T[one]%2T[two]%3T[three]%T%=%999X[X]'
      vim.g.stl_before = vim.fn.tabpagenr()
    ]],
    { { 'left', 'press', '', 0, 6 }, { 'left', 'release', '', 0, 6 } },
    { rows = { 1 } }
  )
  emit('s6/tabs/after', 'A', esc(scrub(ins(lua('return {tab = vim.fn.tabpagenr(), n = vim.fn.tabpagenr("$")}')))))
  click(
    's6/tabs/close',
    "vim.o.tabline = '%1T[one]%2T[two]%3T[three]%T%=%999X[X]'",
    { { 'left', 'press', '', 0, 78 }, { 'left', 'release', '', 0, 78 } },
    { rows = { 1 } }
  )
  emit('s6/tabs/closed', 'A', esc(scrub(ins(lua('return {tab = vim.fn.tabpagenr(), n = vim.fn.tabpagenr("$")}')))))
  lua('vim.cmd("silent! tabonly")')

  -- Malformed and adjacent click definitions.
  local ODD = {
    { 'no-close', '%@StlClick@abc' },
    { 'close-only', 'abc%X' },
    { 'empty-region', '%@StlClick@%X' },
    { 'adjacent', '%@StlClick@a%X%@StlClick2@b%X' },
    { 'nested', '%@StlClick@a%@StlClick2@b%X%X' },
    { 'missing-func', '%@NoSuchClickFn@abc%X' },
    { 'no-at-terminator', '%@StlClickabc' },
    { 'sep-inside', '%@StlClick@a%=b%X' },
    { 'group-inside', '%@StlClick@%(ab%)%X' },
    { 'trunc-inside', '%@StlClick@ab%<cd%X' },
    { 'vlua', '%@v:lua.NoSuch@abc%X' },
  }
  for _, o in ipairs(ODD) do
    click(
      's6/odd/' .. o[1],
      'vim.o.tabline = ' .. string.format('%q', o[2]),
      { { 'left', 'press', '', 0, 1 }, { 'left', 'release', '', 0, 1 } },
      { rows = { 1 } }
    )
  end

  embed_stop(chan)
end)

-- ==================================================================== s7
-- 'statuscolumn'.  `use_statuscol_lnum` reaches the same item loop with
-- a `StatusCol` attached, which is what `%s`, `%C`, `%l` and `%r`
-- read; signs, folds and virtual lines are the three shapes of it.
-- ====================================================================

section('s7-statuscol', function()
  fixture()
  vim.cmd('silent! enew!')
  vim.api.nvim_buf_set_lines(0, 0, -1, false, {
    'one',
    'two',
    'three',
    'four',
    'five',
    'six',
    'seven',
    'eight',
    string.rep('long ', 40),
    'ten',
  })
  local nsign = vim.api.nvim_create_namespace('stlsweep-sign')
  local nvirt = vim.api.nvim_create_namespace('stlsweep-virt')
  vim.api.nvim_buf_set_extmark(0, nsign, 1, 0, { sign_text = 'S1' })
  vim.api.nvim_buf_set_extmark(0, nsign, 1, 0, { sign_text = 'S2', sign_hl_group = 'Todo' })
  vim.api.nvim_buf_set_extmark(0, nsign, 4, 0, { sign_text = '>>' })
  vim.api.nvim_buf_set_extmark(0, nvirt, 2, 0, { virt_lines = { { { 'VIRT-ABOVE', 'Comment' } } }, virt_lines_above = true })
  vim.api.nvim_buf_set_extmark(0, nvirt, 5, 0, { virt_lines = { { { 'VIRT-BELOW', 'Comment' } } } })
  vim.wo.foldmethod = 'manual'
  pcall(vim.cmd, 'silent! 7,8fold')
  park(3, 0)

  local FMTS = {
    { 'lnum-item', '[%l]' },
    { 'relnum-item', '[%r]' },
    { 'sign', '[%s]' },
    { 'fold', '[%C]' },
    { 'all', '[%s][%C][%l][%r]' },
    { 'v-lnum', '[%{v:lnum}]' },
    { 'v-relnum', '[%{v:relnum}]' },
    { 'v-virtnum', '[%{v:virtnum}]' },
    { 'v-all', '[%{v:lnum}/%{v:relnum}/%{v:virtnum}]' },
    { 'cond', '%{v:relnum?v:relnum:v:lnum}' },
    { 'sep', 'a%=b' },
    { 'group', '%(%s%)%(%l%)' },
    { 'width', '%8l|%-8l|%.2l' },
    { 'trunc', 'abcdef%<ghij' },
    { 'sign-twice', '[%s%s]' },
    { 'fold-twice', '[%C%C]' },
    { 'hl', '%#Todo#%l%*' },
    { 'tab-in-statuscol', '[%1Tx]' },
  }

  local CTX = {
    {
      'plain',
      function()
        vim.wo.number = false
        vim.wo.relativenumber = false
        vim.wo.signcolumn = 'no'
        vim.wo.foldcolumn = '0'
      end,
    },
    {
      'number',
      function()
        vim.wo.number = true
        vim.wo.relativenumber = false
        vim.wo.signcolumn = 'no'
        vim.wo.foldcolumn = '0'
      end,
    },
    {
      'relnum',
      function()
        vim.wo.number = false
        vim.wo.relativenumber = true
        vim.wo.signcolumn = 'no'
        vim.wo.foldcolumn = '0'
      end,
    },
    {
      'both-nu',
      function()
        vim.wo.number = true
        vim.wo.relativenumber = true
        vim.wo.signcolumn = 'no'
        vim.wo.foldcolumn = '0'
      end,
    },
    {
      'numwidth8',
      function()
        vim.wo.number = true
        vim.wo.relativenumber = false
        vim.wo.numberwidth = 8
        vim.wo.signcolumn = 'no'
        vim.wo.foldcolumn = '0'
      end,
    },
    {
      'signs-yes',
      function()
        vim.wo.numberwidth = 4
        vim.wo.number = true
        vim.wo.relativenumber = false
        vim.wo.signcolumn = 'yes'
        vim.wo.foldcolumn = '0'
      end,
    },
    {
      'signs-yes3',
      function()
        vim.wo.number = true
        vim.wo.signcolumn = 'yes:3'
        vim.wo.foldcolumn = '0'
      end,
    },
    {
      'signs-auto',
      function()
        vim.wo.number = true
        vim.wo.signcolumn = 'auto:2'
        vim.wo.foldcolumn = '0'
      end,
    },
    {
      'fold1',
      function()
        vim.wo.number = true
        vim.wo.signcolumn = 'no'
        vim.wo.foldcolumn = '1'
      end,
    },
    {
      'fold4',
      function()
        vim.wo.number = true
        vim.wo.signcolumn = 'no'
        vim.wo.foldcolumn = '4'
      end,
    },
    {
      'signs-and-fold',
      function()
        vim.wo.number = true
        vim.wo.signcolumn = 'yes:2'
        vim.wo.foldcolumn = '2'
      end,
    },
  }

  for _, c in ipairs(CTX) do
    c[2]()
    ask('s7/' .. c[1] .. '#ctx', {
      nu = vim.wo.number,
      rnu = vim.wo.relativenumber,
      nuw = vim.wo.numberwidth,
      scl = vim.wo.signcolumn,
      fdc = vim.wo.foldcolumn,
      cursor = vim.api.nvim_win_get_cursor(0),
    })
    for _, f in ipairs(FMTS) do
      for _, lnum in ipairs({ 1, 2, 3, 6, 7, 8, 9, 10 }) do
        ev(
          's7/' .. c[1] .. '/' .. f[1] .. '/l' .. lnum,
          f[2],
          { use_statuscol_lnum = lnum, maxwidth = 60 }
        )
      end
    end
  end

  -- Closed fold, and out-of-range line numbers.
  pcall(vim.cmd, 'silent! 7,8fold')
  pcall(vim.cmd, 'silent! normal! 7Gzc')
  vim.wo.foldcolumn = '3'
  vim.wo.number = true
  for _, lnum in ipairs({ 6, 7, 8, 9 }) do
    ev('s7/closedfold/l' .. lnum, '[%C][%l][%{v:virtnum}]', { use_statuscol_lnum = lnum, maxwidth = 60 })
  end
  for _, lnum in ipairs({ 0, -1, 11, 100, 2147483647 }) do
    ev(
      's7/range/l' .. tostring(lnum):gsub('-', 'neg'),
      '[%l][%s][%C][%{v:lnum}]',
      { use_statuscol_lnum = lnum, maxwidth = 60 }
    )
  end

  -- The `'statuscolumn'` option through a real draw, so the width the
  -- option computes is visible beside the string it produces.
  vim.wo.statuscolumn = '%s%l|'
  ask('s7#option', {
    scl = vim.wo.statuscolumn,
    textoff = vim.fn.getwininfo(vim.api.nvim_get_current_win())[1].textoff,
  })
  vim.wo.statuscolumn = ''
end)

-- ==================================================================== s8
-- 'rulerformat', 'tabline' and 'winbar' over the same alphabet, plus
-- the ruler through a real draw (which only happens with the cursor in
-- a float when 'laststatus' is 2 -- B19-1).
-- ====================================================================

section('s8-others', function()
  fixture()
  vim.cmd('silent! edit ' .. vim.fn.fnameescape(FIXDIR .. '/named.txt'))
  park(3, 2)

  for _, it in ipairs(ITEMS) do
    ev('s8/tabline/' .. it[1], it[2], { use_tabline = true })
    ev('s8/winbar/' .. it[1], it[2], { use_winbar = true })
  end
  ev('s8/tabline#default', '%#TabLineFill#%T%1T one %2T two %=%999Xclose', {
    use_tabline = true,
    highlights = true,
  })
  ev('s8/winbar#default', '%f %m%=%l:%c', { use_winbar = true, highlights = true })
  ev('s8/tabline#width', '%20(a%)%=%20(b%)', { use_tabline = true, maxwidth = 60 })
  ev('s8/tabline#exclusive', '%f', { use_tabline = true, use_winbar = true })

  -- 'rulerformat' as a format string, first.
  local RULERS = {
    { 'default', '%-14.14(%l,%c%V%) %P' },
    { 'plain', '%l,%c' },
    { 'wide', '%40(RULER %l,%c%V %P%)' },
    { 'sep', 'L%=R' },
    { 'expr', '%{line(".")}/%{line("$")}' },
    { 'trunc', 'aaaaaaaaaa%<bbbbbbbbbb' },
    { 'empty', '' },
  }
  for _, r in ipairs(RULERS) do
    ev('s8/ruler/' .. r[1], r[2], { maxwidth = 40 })
  end

  -- ... then through `redraw_ruler`, in a child, with the cursor parked
  -- in a float: `redraw_ruler` returns immediately when the window it
  -- picks has a status line, and with laststatus=2 every ordinary
  -- window has one.  Drop the float and this measures nothing.
  local chan = embed_start()
  local function lua(code)
    local ok, res = pcall(vim.rpcrequest, chan, 'nvim_exec_lua', code, {})
    if ok then
      return res
    end
    return 'RPCERR ' .. errtext(res)
  end
  lua([[
    vim.o.laststatus = 2
    vim.o.ruler = true
    vim.api.nvim_buf_set_lines(0, 0, -1, false, {'alpha','bravo','charlie','delta','echo'})
    vim.api.nvim_win_set_cursor(0, {3, 2})
  ]])
  for _, r in ipairs(RULERS) do
    for _, float in ipairs({ true, false }) do
      local got = lua(string.format(
        [[
        -- `:only` does NOT close a floating window, so without this the
        -- float from the previous scenario survives and every
        -- `nofloat` row reads exactly like its `float` twin -- which is
        -- what the first draft produced, identically, all seven times.
        for _, w in ipairs(vim.api.nvim_list_wins()) do
          if vim.api.nvim_win_get_config(w).relative ~= '' then
            pcall(vim.api.nvim_win_close, w, true)
          end
        end
        vim.cmd('silent! only')
        vim.o.rulerformat = %q
        local fl = %s
        if fl then
          local b = vim.api.nvim_create_buf(false, true)
          vim.api.nvim_buf_set_lines(b, 0, -1, false, {'float-a','float-b','float-c'})
          local w = vim.api.nvim_open_win(b, true, {relative='editor', row=2, col=2, width=20, height=3})
          vim.api.nvim_win_set_cursor(w, {2, 3})
        end
        -- `:mode`, not `:redraw` and not even `:redraw!`.  When the
        -- ruler is NOT drawn nothing clears the command line, so the
        -- previous scenario's ruler sits on row 24 and every `nofloat`
        -- row reads exactly like its `float` twin -- measured,
        -- identically, all seven times.  `:redraw!` does not fix it
        -- (UPD_CLEAR does not reach the message area headless); `:mode`
        -- does, because it clears the screen outright.
        vim.cmd('mode')
        vim.cmd('redraw')
        local out = {}
        for _, r in ipairs({vim.o.lines - 1, vim.o.lines}) do
          local s = {}
          for c = 1, vim.o.columns do s[#s+1] = vim.fn.screenstring(r, c) end
          out[#out+1] = r .. ':' .. (table.concat(s):gsub('%%s+$', ''))
        end
        return out
      ]],
        r[2],
        tostring(float)
      ))
      local label = 's8/rulerdraw/' .. r[1] .. (float and '/float' or '/nofloat')
      label_once(label)
      emit(label, '=', esc(scrub(ins(got))))
      struct(label, got)
    end
  end

  -- 'tabline' and 'winbar' through a real draw too.
  for _, t in ipairs({
    { 'plain', 'TAB %{bufnr()} %=end' },
    { 'wide', '%50(pad%)%=x' },
    { 'trunc', string.rep('abcdefghij', 12) .. '%<TAIL' },
    { 'hl', '%#Todo#a%#Comment#b%*c' },
  }) do
    local got = lua(string.format(
      [[
      vim.cmd('silent! only')
      vim.o.showtabline = 2
      vim.o.tabline = %q
      vim.o.winbar = %q
      vim.cmd('mode')
      vim.cmd('redraw')
      local out = {}
      for _, r in ipairs({1, 2}) do
        local s = {}
        for c = 1, vim.o.columns do s[#s+1] = vim.fn.screenstring(r, c) end
        out[#out+1] = r .. ':' .. (table.concat(s):gsub('%%s+$', ''))
      end
      return out
    ]],
      t[2],
      t[2]
    ))
    local label = 's8/tabdraw/' .. t[1]
    label_once(label)
    emit(label, '=', esc(scrub(ins(got))))
    struct(label, got)
  end

  -- 'statusline' through a real draw, including the global one.
  for _, t in ipairs({
    { 'local', 'set laststatus=2', 'STAT %{bufnr()}%=%l,%c' },
    { 'global', 'set laststatus=3', 'GLOBAL %{bufnr()}%=%l,%c' },
    { 'trunc', 'set laststatus=2', string.rep('xy', 60) .. '%<CUT' },
    { 'nc', 'set laststatus=2', 'NC %{winnr()}%=%f' },
  }) do
    local got = lua(string.format(
      [[
      vim.cmd('silent! only')
      vim.cmd(%q)
      vim.o.showtabline = 0
      vim.o.winbar = ''
      vim.o.statusline = %q
      vim.cmd('silent! split')
      vim.cmd('mode')
      vim.cmd('redraw')
      local out = {}
      for _, r in ipairs({vim.o.lines - 1, math.floor(vim.o.lines / 2)}) do
        local s = {}
        for c = 1, vim.o.columns do s[#s+1] = vim.fn.screenstring(r, c) end
        out[#out+1] = r .. ':' .. (table.concat(s):gsub('%%s+$', ''))
      end
      vim.cmd('silent! only')
      return out
    ]],
      t[2],
      t[3]
    ))
    local label = 's8/statdraw/' .. t[1]
    label_once(label)
    emit(label, '=', esc(scrub(ins(got))))
    struct(label, got)
  end

  embed_stop(chan)
end)

-- ==================================================================== s9
-- Malformed formats and the message path.
-- ====================================================================

local BAD = {
  { 'trailing-pct', 'abc%' },
  { 'pct-nul', '%' },
  { 'pct-space', '% ' },
  { 'unknown-Z', '%Z' },
  { 'unknown-tilde', '%~' },
  { 'unknown-slash', '%/' },
  { 'unknown-quote', "%'" },
  { 'unknown-bang-mid', 'x%!1' },
  { 'unknown-high', '%\200' },
  { 'open-group', '%(abc' },
  { 'close-group', 'abc%)' },
  { 'close-then-open', '%)%(' },
  { 'group-two-open', '%(%(a%)' },
  { 'group-two-close', '%(a%)%)' },
  { 'open-brace', '%{' },
  { 'open-brace-text', '%{abc' },
  { 'brace-close-only', 'abc}' },
  { 'reeval-unclosed', "%{%'a'" },
  { 'reeval-half', '%{%' },
  { 'reeval-wrong-close', "%{%'a'}" },
  { 'hl-unterminated', '%#Todo' },
  { 'hl-empty', '%##' },
  { 'hl-space', '%# #' },
  { 'comb-unterminated', '%$Todo' },
  { 'comb-empty', '%$$' },
  { 'click-no-at', '%@Foo' },
  { 'click-no-body', '%@Foo@' },
  { 'click-empty-name', '%@@x%X' },
  { 'width-only', '%5' },
  { 'width-neg-only', '%-5' },
  { 'dot-only', '%.' },
  { 'dot-then-pct', '%.%' },
  { 'digits-only', '%12345' },
  { 'minus-only', '%-' },
  { 'minus-dot', '%-.' },
  { 'star-neg', '%-1*' },
  { 'star-huge', '%2147483648*' },
  { 'T-no-arg', '%T' },
  { 'X-no-arg', '%X' },
  { 'nul-in-format', 'a\0b' },
  { 'group-in-brace', '%{%(}' },
  { 'brace-in-group', '%(%{1%)}' },
  { 'trunc-then-pct', '%<%' },
  { 'sep-then-pct', '%=%' },
  { 'lone-pct-pct-pct', '%%%' },
}

section('s9-errors', function()
  fixture()
  vim.cmd('silent! edit ' .. vim.fn.fnameescape(FIXDIR .. '/named.txt'))
  park(3, 2)
  for _, b in ipairs(BAD) do
    ev('s9/bad/' .. b[1], b[2])
    ev('s9/bad-tab/' .. b[1], b[2], { use_tabline = true })
    ev('s9/bad-col/' .. b[1], b[2], { use_statuscol_lnum = 2 })
  end

  -- Bad option arguments to the API itself.
  local OPTS = {
    { 'winid-bad', 'x', { winid = 99999 } },
    { 'winid-zero', 'x', { winid = 0 } },
    { 'tabline-and-winbar', 'x', { use_tabline = true, use_winbar = true } },
    { 'fillchar-two', 'a%=b', { fillchar = 'ab', maxwidth = 10 } },
    { 'fillchar-empty', 'a%=b', { fillchar = '', maxwidth = 10 } },
    { 'fillchar-wide', 'a%=b', { fillchar = '\228\184\128', maxwidth = 10 } },
    { 'fillchar-nul', 'a%=b', { fillchar = '\0', maxwidth = 10 } },
    { 'maxwidth-neg', 'abcdef', { maxwidth = -5 } },
    { 'maxwidth-zero', 'abcdef', { maxwidth = 0 } },
    { 'statuscol-and-tabline', '%l', { use_statuscol_lnum = 1, use_tabline = true } },
  }
  for _, o in ipairs(OPTS) do
    ev('s9/opt/' .. o[1], o[2], o[3])
  end

  -- THE MESSAGE PATH.  In process, a Vimscript error inside a `%{}`
  -- becomes a Lua error the pcall swallows and nvim never *displays*
  -- anything, so the .stderr artifact comes out empty while every other
  -- one looks healthy.  `-c` in a child is the only spelling that
  -- prints and keeps going -- and nvim caps `-c`/`--cmd` at TEN in
  -- total, answering `Too many "+command"` on stderr (i.e. into the
  -- artifact), so this chunks at five.
  local CMDS = {
    'set laststatus=2 statusline=%{NoSuchStlFunc()}|redraw',
    'set laststatus=2 statusline=%{throw_here|redraw',
    'set laststatus=2 statusline=%Z|redraw',
    'set laststatus=2 statusline=%(abc|redraw',
    'set laststatus=2 statusline=abc%)|redraw',
    'set laststatus=2 statusline=%!NoSuchStlFunc()|redraw',
    'set showtabline=2 tabline=%{NoSuchStlFunc()}|redraw',
    'set showtabline=2 tabline=%(|redraw',
    'set winbar=%{NoSuchStlFunc()}|redraw',
    'set rulerformat=%(|set ruler|redraw',
    'set statuscolumn=%{NoSuchStlFunc()}|redraw',
    'set statuscolumn=%(|redraw',
    'echo nvim_eval_statusline("%Z", {})',
    'echo nvim_eval_statusline("%(", {})',
    'echo nvim_eval_statusline("%{", {})',
    'echo nvim_eval_statusline("x", {"winid": 99999})',
    'echo nvim_eval_statusline("x", {"use_tabline": v:true, "use_winbar": v:true})',
    'echo nvim_eval_statusline("x", {"fillchar": "ab"})',
    'echo nvim_eval_statusline([], {})',
    'echo nvim_eval_statusline("x", [])',
  }
  local codes = {}
  local at = 1
  while at <= #CMDS do
    local args = { work .. '/bin/nvim', '--headless', '-u', 'NONE', '-i', 'NONE' }
    local upto = math.min(at + 4, #CMDS)
    for i = at, upto do
      args[#args + 1] = '-c'
      args[#args + 1] = CMDS[i]
    end
    args[#args + 1] = '-c'
    args[#args + 1] = 'qa!'
    local res = vim
      .system(args, {
        text = true,
        cwd = work,
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
        clear_env = true,
        timeout = 60000,
      })
      :wait()
    io.stderr:write('\n-- s9 chunk ' .. at .. '\n' .. scrub(res.stderr or ''))
    codes[#codes + 1] = tostring(res.code)
    at = upto + 1
  end
  emit('s9', 'A', string.format('cmds=%d codes=%s', #CMDS, table.concat(codes, ',')))
  struct('s9#messages', { cmds = #CMDS, codes = codes })
end)

-- =================================================================== s91
-- CRASHPROBE.  One child per input.
-- ====================================================================

section('s91-crashprobe', function()
  local progpath = work .. '/bin/nvim'
  local env = {
    HOME = work .. '/home',
    PATH = work .. '/bin',
    TMPDIR = work .. '/tmp',
    TERM = 'dumb',
    SHELL = '/bin/sh',
    LANG = 'C.UTF-8',
    VIMRUNTIME = runtime,
    NVIM_TEST = '1',
    STL_WORK = work,
  }
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
      }, { text = true, cwd = work, env = env, clear_env = true, timeout = 120000 })
      :wait()
    local last = i - 1
    for line in (res.stdout or ''):gmatch('[^\n]+') do
      local idx = tonumber(line:match('^(%d+) '))
      if idx then
        last = idx
        emit((line:gsub('^%d+ ', '')))
      end
    end
    -- vim.system reports an abort as a SIGNAL and leaves `code` at 0.
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
