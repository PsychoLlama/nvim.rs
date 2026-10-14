-- Driver for the read-base oracle; see readbase.sh.
--
--   nvim --headless -u NONE -i NONE -l readbase.lua <kind> <artifact> [<text>]
--
-- Reads ONE on-disk artifact some other binary wrote -- a scrubbed `.swp`
-- (kind `swap`), a version-masked `.shada` (kind `shada`) or a scrubbed
-- `.un~` plus the text its tree hashes (kind `undo`) -- and prints what the
-- editor decoded from it.  One process per artifact, so nothing one read
-- leaves behind (a register, a history entry, a mark) reaches the next.
--
-- The `pers` and `undo` rows only ever read back what the binary under
-- test just wrote, so a reader and a writer that drifted together would
-- pass both.  This one reads bytes a *fixed* binary wrote.
--
-- Everything printed has to be a function of the artifact alone: the work
-- directory is masked, and no time, pid or host is printed.

local kind, artifact, text = arg[1], arg[2], arg[3]
local work = assert(os.getenv('READBASE_WORK'), 'READBASE_WORK unset')

local function emit(...)
  io.write(table.concat({ ... }, ' '), '\n')
end

--- Escape to one printable line, so a byte difference shows in the diff.
local function esc(bytes)
  return (
    tostring(bytes):gsub('[^\32-\126]', function(c)
      return string.format('\\x%02x', c:byte())
    end)
  )
end

local function scrub(s)
  s = tostring(s):gsub(vim.pesc(work), '<WORK>')
  return s
end

--- Run an Ex command, returning its messages (or its error) scrubbed.
local function exec(cmd)
  local ok, out = pcall(vim.fn.execute, cmd)
  if not ok then
    out = 'ERROR ' .. tostring(out)
  end
  local lines = {}
  for line in (scrub(out) .. '\n'):gmatch('([^\n]*)\n') do
    if line ~= '' then
      lines[#lines + 1] = esc(line)
    end
  end
  return lines
end

local function report(label, cmd)
  for _, line in ipairs(exec(cmd)) do
    emit(' ', label, '|', line)
  end
end

--- Canonical, key-sorted dump of a Vimscript value.
local function canon(v)
  if type(v) == 'table' then
    if vim.islist(v) then
      local parts = {}
      for i, item in ipairs(v) do
        parts[i] = canon(item)
      end
      return '[' .. table.concat(parts, ',') .. ']'
    end
    local names = vim.tbl_keys(v)
    table.sort(names)
    local parts = {}
    for i, name in ipairs(names) do
      parts[i] = name .. '=' .. canon(v[name])
    end
    return '{' .. table.concat(parts, ' ') .. '}'
  end
  return esc(scrub(v))
end

--- The whole buffer: its size, a digest, and the lines themselves when
--- few enough to read.
local function buffer(label)
  local lines = vim.api.nvim_buf_get_lines(0, 0, -1, true)
  emit(
    ' ',
    label,
    'lines',
    #lines,
    'modified',
    tostring(vim.bo.modified),
    'ff',
    vim.bo.fileformat,
    'fenc',
    vim.bo.fileencoding,
    'sha',
    vim.fn.sha256(table.concat(lines, '\n'))
  )
  if #lines <= 40 then
    for i, line in ipairs(lines) do
      emit(' ', label, i, '|', #line <= 200 and esc(line) or (esc(line:sub(1, 100)) .. '... ' .. #line .. ' bytes'))
    end
  end
end

vim.o.swapfile = false
vim.o.shortmess = ''
vim.o.more = false

if kind == 'swap' then
  vim.o.directory = work
  report('recover', 'recover ' .. vim.fn.fnameescape(artifact))
  emit('  name', esc(scrub(vim.api.nvim_buf_get_name(0))))
  buffer('buffer')
elseif kind == 'shada' then
  -- Every item kind on, so the read keeps whatever the file carries.
  vim.o.shada = "!,'100,<100,s100,h,%,:100,/100,@100"
  report('rshada', 'rshada! ' .. vim.fn.fnameescape(artifact))
  local regs = {}
  for _, r in ipairs(vim.split('"0123456789abcdefghijklmnopqrstuvwxyz-', '')) do
    local value = vim.fn.getreg(r, 1, true)
    if #value > 0 then
      regs[#regs + 1] = r .. ':' .. vim.fn.getregtype(r) .. '=' .. canon(value)
    end
  end
  emit('  registers', table.concat(regs, ' '))
  for _, h in ipairs({ 'cmd', 'search', 'expr', 'input', 'debug' }) do
    local entries = {}
    for i = 1, vim.fn.histnr(h) do
      local entry = vim.fn.histget(h, i)
      if entry ~= '' then
        entries[#entries + 1] = esc(entry)
      end
    end
    emit('  history', h, table.concat(entries, ' | '))
  end
  emit('  search', esc(vim.fn.getreg('/')))
  emit('  marks', canon(vim.fn.getmarklist()))
  emit('  oldfiles', canon(vim.v.oldfiles))
  local vars = {}
  for name, value in pairs(vim.fn.eval('g:')) do
    if name:match('^%u[%u%d_]*$') then
      vars[#vars + 1] = name .. '=' .. canon(value)
    end
  end
  table.sort(vars)
  emit('  vars', table.concat(vars, ' '))
  -- And straight back out, unmerged: the shell tokenises this file, so
  -- what the reader kept of every entry (jumps, changes, the buffer
  -- list, the substitute string -- none of which a function reads) is
  -- seen too.
  report('wshada', 'wshada! ' .. vim.fn.fnameescape(work .. '/rewrite.shada'))
elseif kind == 'undo' then
  -- The tree hashes its buffer's text, so the read needs that text: a
  -- copy of it, under the work directory, edited fresh.
  local file = work .. '/f.txt'
  vim.fn.writefile(vim.fn.readblob(text), file, 'b')
  vim.cmd('silent edit! ' .. vim.fn.fnameescape(file))
  report('rundo', 'rundo ' .. vim.fn.fnameescape(artifact))
  local tree = vim.fn.undotree()
  emit('  tree', canon(tree))
  -- Every state the tree holds, by sequence number: `:undo N` walks
  -- there through whatever branches the file recorded.
  for seq = 0, tree.seq_last do
    report('undo ' .. seq, 'undo ' .. seq)
    buffer('seq ' .. seq)
  end
else
  error('readbase: unknown kind ' .. tostring(kind))
end
