-- Driver for the undofile golden oracle; see undogold.sh.
--
-- Covers the one on-disk surface batch B-undo owns: the `.un~` file that
-- `undo/write.rs` + `undo/file.rs` lay down.  Every case builds a
-- deterministic undo tree in a fresh buffer and then forces the tree out
-- with `:wundo!` (and, for two cases, through the 'undofile' auto-write
-- path, which is the same `u_write_undo` with `name == NULL`).
--
-- Two layers come out of this:
--
--   the report (stdout)   undotree(), :undolist and the buffer text --
--                         what the tree *means*
--   the artifacts         the produced `.un~` bytes, copied into
--                         $SWEEP_ART for undoscrub.py to
--                         decode, mask and hash
--
-- Everything printed has to be reproducible across two builds run
-- minutes apart, so the report never carries a path outside the work
-- directory and never a wall-clock time.  The undo tree records time in
-- three places -- `undotree().time_cur`, every entry's `time`, and the
-- `:undolist` "time" column -- and all three are masked here.  The
-- corresponding *file* fields (`b_u_time_cur`, `uh_time`) are masked by
-- the scrubber, not here.

local work = assert(os.getenv('SWEEP_WORK'), 'SWEEP_WORK unset')
local art = assert(os.getenv('SWEEP_ART'), 'SWEEP_ART unset')
local undodir = work .. '/undo'
local cases = work .. '/cases'

local function emit(...)
  io.write(table.concat({ ... }, ' '), '\n')
end

local function slurp(path)
  local fd = io.open(path, 'rb')
  if not fd then
    return nil
  end
  local bytes = fd:read('*a')
  fd:close()
  return bytes
end

local function spew(path, bytes)
  local fd = assert(io.open(path, 'wb'))
  fd:write(bytes)
  -- Explicitly, not on collection: the scrubber runs in a separate
  -- process the moment nvim exits, and an unflushed copy would read as a
  -- truncated file.
  fd:close()
end

--- Escape to one printable line, so a byte difference shows in the diff.
local function esc(bytes)
  return (tostring(bytes):gsub('[^\32-\126]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

--- Strip the bits of a message that name where the run happened.
local function scrub(text)
  text = tostring(text)
  text = text:gsub(vim.pesc(work), '<WORK>')
  -- Messages are truncated to the (headless, 80 column) screen width,
  -- which lops the leading characters off a long path; the tail still has
  -- to be recognised.
  text = text:gsub(vim.pesc(work:sub(2)), '<WORK>')
  -- `undodir` munges every path separator to '%', so the same path
  -- appears in a second spelling.
  text = text:gsub(vim.pesc((work:gsub('/', '%%'))), '<WORKPCT>')
  -- `:earlier`/`:later` report how long ago the state they landed on
  -- was current.  A slow run turns "0 seconds ago" into "1 second ago".
  text = text:gsub('%d+ %a+s? ago', '<AGO>')
  text = text:gsub('%s+\n', '\n'):gsub('%s+$', '')
  return text
end

local function exec(cmd)
  local ok, res = pcall(vim.api.nvim_exec2, cmd, { output = true })
  if ok then
    return scrub(res.output or '')
  end
  -- Errors are observable behaviour: a case that fails has to fail the
  -- same way in both binaries.
  return 'ERROR ' .. scrub(res)
end

--- Run `cmd`, print its output under a heading only when it said something.
local function run(label, cmd)
  local out = exec(cmd)
  if out ~= '' then
    emit(' ', label, '|', (out:gsub('\n', '\n  ' .. label .. ' | ')))
  else
    emit(' ', label, '| (silent)')
  end
end

--- Feed `s` as if typed, and let it finish before returning.
--
-- Whatever it said goes into the report.  A fed keystroke's messages are
-- not captured by `nvim_exec2` the way an `:ex` command's are -- they go
-- straight out -- so the message history is cleared first and read back
-- after, which is the only way to attribute a message to a keystroke.
local function keys(s)
  vim.cmd('silent messages clear')
  local t = vim.api.nvim_replace_termcodes(s, true, false, true)
  vim.api.nvim_feedkeys(t, 'nx', false)
  local said = scrub(vim.fn.execute('messages')):gsub('^%s+', '')
  emit('  keys', esc(s), '|', said == '' and '(silent)' or (said:gsub('\n', ' / ')))
end

-- =========================================================================
-- The behaviour layer
-- =========================================================================

-- `undotree()` and `:undolist` both report wall-clock time.  The *shape*
-- of the tree is the thing under test, so the values go and the keys
-- stay: a field that stopped being reported is still a report
-- difference.
local TIME_KEYS = { time = true, time_cur = true }

--- Canonical, key-sorted dump of an undotree() value.
local function canon(v, key)
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
      parts[i] = name .. '=' .. canon(v[name], name)
    end
    return '{' .. table.concat(parts, ' ') .. '}'
  end
  if key and TIME_KEYS[key] then
    return '<TIME>'
  end
  return esc(tostring(v))
end

--- `:undolist`, with its "time" column masked.
local function undolist()
  local out = exec('undolist')
  local lines = {}
  for line in (out .. '\n'):gmatch('([^\n]*)\n') do
    -- "  number changes  when               saved"; `when` is either a
    -- clock time or an "N seconds ago" phrase, and both move between runs.
    local number, changes, rest =
      line:match('^%s*(%d+)%s+(%d+)%s+%d%d:%d%d:%d%d%s*(.*)$')
    if not number then
      -- `scrub` has already turned the phrase into <AGO> by the time the
      -- line gets here; the clock spelling above is what a change older
      -- than a hundred seconds would print instead, and both have to
      -- normalise to the same text or a slow run reads as a difference.
      number, changes, rest =
        line:match('^%s*(%d+)%s+(%d+)%s+<AGO>%s*(.*)$')
    end
    if number then
      line = ('  %s %s <WHEN> %s'):format(number, changes, rest)
    end
    lines[#lines + 1] = (line:gsub('%s+$', ''))
  end
  return table.concat(lines, '\n')
end

--- The full behaviour snapshot of the current buffer's undo state.
local function snapshot(label)
  emit(' ', label, 'lines |', esc(table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, true), '\\n')))
  emit(' ', label, 'tree  |', canon(vim.fn.undotree()))
  emit(' ', label, 'list  |', (undolist():gsub('\n', '\n  ' .. label .. ' list  | ')))
end

-- =========================================================================
-- Case scaffolding
-- =========================================================================

local kept = 0

--- Copy a produced file into the artifact tree under a flat name.
local function keep(name, path)
  local bytes = slurp(path)
  if not bytes then
    emit('  keep', name, 'MISSING')
    return false
  end
  spew(art .. '/' .. name, bytes)
  kept = kept + 1
  emit('  keep', name, #bytes, 'bytes')
  return true
end

local index = 0

-- The undo level in force for the current case, tracked here because
-- `sync` below has to re-set the option to whatever it already is and
-- reading it back off the buffer returns a sentinel when it is unset.
local ul = 1000

--- Close off the pending change, so the next edit gets a header of its own.
--
-- A scripted run never returns to the main loop, so nothing calls
-- `u_sync` on its own and every edit in the script would land in one
-- single header.  Re-setting 'undolevels' is the documented way to force
-- the boundary: `did_set_undolevels` calls `u_sync(true)` under the old
-- limit before installing the new one, and it runs whether or not the
-- value actually changed.
local function sync()
  vim.cmd('setlocal undolevels=' .. tostring(ul))
end

--- Change the undo level for this case, and take the boundary with it.
local function setul(n)
  ul = n
  vim.cmd('setlocal undolevels=' .. tostring(n))
end

--- Fresh buffer, fresh file, fixed options.  Returns the file's path.
--
-- Each case gets its own directory: the undo file's name is derived from
-- the edited file's name, and a collision would make case N depend on
-- case N-1.
local function fresh(name, lines)
  vim.cmd('silent! %bwipeout!')
  local dir = ('%s/%s'):format(cases, name)
  vim.fn.mkdir(dir, 'p')
  local file = dir .. '/f.txt'
  -- Options that reach the file: 'undolevels' bounds the tree,
  -- 'undofile' decides whether `:w` writes one at all, and 'fsync' is
  -- reset because a failed fsync flips `write_ok` and changes the report.
  vim.o.undolevels = 1000
  vim.o.undodir = undodir
  vim.o.undofile = false
  vim.o.fsync = false
  vim.o.swapfile = false
  vim.o.backup = false
  vim.o.writebackup = false
  vim.o.shada = ''
  vim.o.fileformat = 'unix'
  vim.o.fileencoding = ''
  vim.o.binary = false
  vim.o.expandtab = false
  vim.o.virtualedit = ''
  -- The seed content is laid down from Lua rather than by editing and
  -- saving a buffer: a `:write` of an edited buffer leaves an undo tree
  -- behind, and every case has to start from `b_u_numhead == 0` so that
  -- what lands in the file is only what the case itself did.
  spew(file, lines and (table.concat(lines, '\n') .. '\n') or '')
  vim.cmd('edit! ' .. vim.fn.fnameescape(file))
  -- `setlocal` and not `vim.bo`: 'undolevels' is global-local, and a
  -- previous case may have left the local value somewhere else.
  setul(1000)
  return file
end

--- Force the tree out to an explicit name and keep the bytes.
local function wundo(name)
  index = index + 1
  local out = ('%s/%02d-%s.un~'):format(cases, index, name)
  run('wundo', 'wundo! ' .. vim.fn.fnameescape(out))
  keep(('%02d-%s.un~'):format(index, name), out)
  return out
end

local function case(name, fn)
  emit('')
  emit(('== case %s'):format(name))
  local ok, err = pcall(fn)
  if not ok then
    emit('  FAILED', scrub(err))
  end
end

-- =========================================================================
-- The cases
-- =========================================================================

vim.fn.delete(work .. '/undo', 'rf')
vim.fn.delete(cases, 'rf')
vim.fn.mkdir(undodir, 'p')
vim.fn.mkdir(cases, 'p')

emit('undofile golden oracle -- undodir <WORK>/undo')

-- 1. The plainest possible tree: one linear chain, no branches.
case('linear', function()
  fresh('linear', { 'one', 'two', 'three' })
  for i = 1, 6 do
    vim.api.nvim_buf_set_lines(0, 0, 0, true, { ('added %d'):format(i) })
    sync()
  end
  snapshot('after')
  wundo('linear')
end)

-- 2. Entries that span several lines: `ue_size > 1`, and `ue_top`/`ue_bot`
--    away from the buffer's edges.
case('multiline', function()
  local lines = {}
  for i = 1, 12 do
    lines[i] = ('line %d'):format(i)
  end
  fresh('multiline', lines)
  run('delete', '3,6delete')
  sync()
  run('normal', 'normal! 2GP')
  sync()
  vim.api.nvim_buf_set_lines(0, 4, 4, true, { 'inserted a', 'inserted b', 'inserted c' })
  sync()
  run('move', '5,7move 1')
  sync()
  snapshot('after')
  wundo('multiline')
end)

-- 3. Branching: undo back into the chain, then edit again, so the old
--    tip becomes an alt branch (`uh_alt_next`/`uh_alt_prev`).
case('branch', function()
  fresh('branch', { 'base' })
  for i = 1, 4 do
    vim.api.nvim_buf_set_lines(0, -1, -1, true, { ('trunk %d'):format(i) })
    sync()
  end
  run('undo', 'undo | undo')
  vim.api.nvim_buf_set_lines(0, -1, -1, true, { 'branch A' })
  sync()
  run('undo', 'undo')
  vim.api.nvim_buf_set_lines(0, -1, -1, true, { 'branch B' })
  sync()
  run('undo', 'undo | undo')
  vim.api.nvim_buf_set_lines(0, -1, -1, true, { 'branch C' })
  sync()
  snapshot('after')
  wundo('branch')
end)

-- 4. `:earlier`/`:later` across a branchy tree -- the time-travel walk,
--    which moves `b_u_seq_cur`/`b_u_curhead` rather than the shape.
case('earlier-later', function()
  fresh('earlier-later', { 'seed' })
  for i = 1, 5 do
    vim.api.nvim_buf_set_lines(0, -1, -1, true, { ('step %d'):format(i) })
    sync()
  end
  run('earlier3', 'earlier 3')
  vim.api.nvim_buf_set_lines(0, -1, -1, true, { 'sidestep' })
  sync()
  snapshot('mid')
  run('earlier2', 'earlier 2')
  snapshot('earlier2')
  run('later1', 'later 1')
  snapshot('later1')
  run('earlier1f', 'earlier 1f')
  snapshot('earlier1f')
  run('laterf', 'later 1f')
  snapshot('laterf')
  wundo('earlier-later')
end)

-- 5. `g-`/`g+`: chronological travel, which visits branches `u`/`CTRL-R`
--    cannot reach.  The buffer text after each step is the behaviour
--    half; the tree it leaves behind is the byte half.
case('gminus', function()
  fresh('gminus', { 'g0' })
  for i = 1, 3 do
    vim.api.nvim_buf_set_lines(0, -1, -1, true, { ('main %d'):format(i) })
    sync()
  end
  run('undo', 'undo | undo')
  vim.api.nvim_buf_set_lines(0, -1, -1, true, { 'alt 1' })
  sync()
  vim.api.nvim_buf_set_lines(0, -1, -1, true, { 'alt 2' })
  sync()
  for i = 1, 7 do
    keys('g-')
    emit(('  g- %d seq=%s | %s'):format(
      i,
      vim.fn.undotree().seq_cur,
      esc(table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, true), '\\n'))
    ))
  end
  for i = 1, 7 do
    keys('g+')
    emit(('  g+ %d seq=%s | %s'):format(
      i,
      vim.fn.undotree().seq_cur,
      esc(table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, true), '\\n'))
    ))
  end
  snapshot('after')
  wundo('gminus')
end)

-- 6. `:undojoin`: two changes that share one header, so the tree has
--    fewer headers than there were changes.
case('undojoin', function()
  fresh('undojoin', { 'a', 'b', 'c' })
  run('normal', 'normal! ggIX')
  sync()
  -- Without the `:undojoin` each of these would open a header of its
  -- own, exactly as the un-joined change at the end does.
  run('undojoin', 'undojoin')
  run('normal', 'normal! jIY')
  run('undojoin', 'undojoin')
  run('normal', 'normal! jIZ')
  sync()
  run('normal', 'normal! GoW')
  sync()
  snapshot('after')
  wundo('undojoin')
end)

-- 7. Blockwise (CTRL-V) insert, delete and change: one keystroke, one
--    header, many entries.
case('blockwise', function()
  local lines = {}
  for i = 1, 6 do
    lines[i] = ('col%dcol'):format(i)
  end
  fresh('blockwise', lines)
  keys('gg0<C-v>3jIBLK<Esc>')
  sync()
  keys('gg0<C-v>3j2ld')
  sync()
  keys('gg0<C-v>5j$AEND<Esc>')
  sync()
  keys('2G0<C-v>2j2lcQQ<Esc>')
  sync()
  snapshot('after')
  wundo('blockwise')
end)

-- 8. Extmarks: `serialize_extmark` only runs when a header carries an
--    `uh_extmark` vector, which happens when a namespace has marks in the
--    changed region.  `:move` is the one edit that produces a
--    `kExtmarkMove` object rather than a `kExtmarkSplice`.
case('extmark', function()
  local lines = {}
  for i = 1, 10 do
    lines[i] = ('mark line %d'):format(i)
  end
  fresh('extmark', lines)
  local ns = vim.api.nvim_create_namespace('undogold')
  for i = 0, 9 do
    vim.api.nvim_buf_set_extmark(0, ns, i, 0, { end_row = i, end_col = 4, hl_group = 'Comment' })
  end
  vim.api.nvim_buf_set_lines(0, 2, 4, true, { 'spliced' })
  sync()
  vim.api.nvim_buf_set_text(0, 0, 2, 0, 6, { 'XY' })
  sync()
  run('move', '4,5move 0')
  sync()
  vim.api.nvim_buf_set_lines(0, -1, -1, true, { 'tail one', 'tail two' })
  sync()
  emit('  extmarks |', canon(vim.api.nvim_buf_get_extmarks(0, ns, 0, -1, {})))
  snapshot('after')
  wundo('extmark')
end)

-- 9. A non-empty Visual selection at the time of the change, which is
--    what puts something in `uh_visual` (`serialize_visualinfo`).  All
--    three Visual modes, because `vi_mode` distinguishes them and
--    blockwise also sets `vi_curswant`.
case('visual', function()
  local lines = {}
  for i = 1, 8 do
    lines[i] = ('visual line %d'):format(i)
  end
  fresh('visual', lines)
  keys('gg0vjjlx')
  sync()
  keys('3GVjd')
  sync()
  keys('2G0<C-v>jj$Ax<Esc>')
  sync()
  -- Leave a live Visual area behind: `u_savecommon` copies
  -- `VIsual`/`curwin->w_cursor` into the *next* header's uh_visual.
  keys('gg0vjl<Esc>')
  vim.api.nvim_buf_set_lines(0, 0, 0, true, { 'after visual' })
  sync()
  emit('  visualmarks |', canon({ vim.fn.getpos("'<"), vim.fn.getpos("'>") }))
  snapshot('after')
  wundo('visual')
end)

-- 10. Named marks: `uh_namedm[NMARKS]` is 26 positions, written for every
--     header whether or not a mark is set.
case('namedm', function()
  local lines = {}
  for i = 1, 30 do
    lines[i] = ('m %d'):format(i)
  end
  fresh('namedm', lines)
  for i = 0, 25 do
    local letter = string.char(97 + i)
    vim.api.nvim_buf_set_mark(0, letter, i + 1, i % 3, {})
  end
  vim.api.nvim_buf_set_lines(0, 4, 6, true, { 'shifted' })
  sync()
  vim.api.nvim_buf_set_lines(0, 0, 0, true, { 'top' })
  sync()
  emit('  marks |', canon(vim.fn.getmarklist(vim.api.nvim_get_current_buf())))
  snapshot('after')
  wundo('namedm')
end)

-- 11. `undolevels=1`: `u_undo_end`'s truncation runs, so the oldest
--     headers are freed and the file holds a stub of the tree.
case('levels1', function()
  fresh('levels1', { 'trunc' })
  setul(1)
  for i = 1, 8 do
    vim.api.nvim_buf_set_lines(0, -1, -1, true, { ('cut %d'):format(i) })
    sync()
  end
  snapshot('after')
  wundo('levels1')
end)

-- 12. `undolevels=0`: exactly one level, i.e. every change frees the
--     previous header.
case('levels0', function()
  fresh('levels0', { 'zero' })
  setul(0)
  for i = 1, 4 do
    vim.api.nvim_buf_set_lines(0, -1, -1, true, { ('zed %d'):format(i) })
    sync()
  end
  snapshot('after')
  wundo('levels0')
end)

-- 13. `undolevels=-1`: undo is off, so no header is ever created and
--     `u_write_undo` takes its "nothing to undo" exit.
case('levels-off', function()
  fresh('levels-off', { 'off' })
  setul(-1)
  for i = 1, 3 do
    vim.api.nvim_buf_set_lines(0, -1, -1, true, { ('none %d'):format(i) })
    sync()
  end
  snapshot('after')
  wundo('levels-off')
end)

-- 14. An empty tree: nothing was changed at all.  `u_write_undo` says so
--     and writes no file, which is itself a behaviour to pin.
case('empty', function()
  fresh('empty', { 'untouched' })
  snapshot('after')
  wundo('empty')
end)

-- 15. `b_u_line_ptr`: the `U` command's saved line lives in the file
--     header, not in any undo header, and is the header's one
--     variable-length field.
case('lineptr', function()
  fresh('lineptr', { 'aaa', 'bbbbbbbb', 'ccc' })
  keys('2G0x')
  keys('x')
  keys('x')
  -- No `sync()` here: `b_u_line_ptr` holds the line as it was before the
  -- *current* run of changes on it, and a boundary would not clear it.
  emit('  Uline |', esc(vim.api.nvim_buf_get_lines(0, 1, 2, true)[1]))
  snapshot('after')
  wundo('lineptr')
end)

-- 16. Save numbers: `b_u_save_nr_last` and each header's `uh_save_nr`
--     are the file header's and the headers' optional-field payloads.
case('savenr', function()
  fresh('savenr', { 's' })
  for i = 1, 3 do
    vim.api.nvim_buf_set_lines(0, -1, -1, true, { ('save %d'):format(i) })
    sync()
    run('write', 'silent write')
  end
  vim.api.nvim_buf_set_lines(0, -1, -1, true, { 'dirty' })
  sync()
  snapshot('after')
  wundo('savenr')
end)

-- 17. Text the length fields have to survive: empty lines, tabs, high
--     bytes and multibyte.  (Not NUL -- `serialize_uep` measures with
--     strlen, so a NUL in a line truncates it; see the learnings.)
case('bytes', function()
  fresh('bytes', {
    '',
    'tab\there',
    'latin1 \xc3\xa9\xc3\xbc\xc3\x9f',
    'cjk \xe6\x97\xa5\xe6\x9c\xac\xe8\xaa\x9e',
    'emoji \xf0\x9f\x98\x80',
    string.rep('w', 300),
    '',
  })
  vim.api.nvim_buf_set_lines(0, 0, 1, true, { 'no longer empty' })
  sync()
  vim.api.nvim_buf_set_lines(0, 3, 3, true, { '', '', '' })
  sync()
  run('delete', '$delete')
  sync()
  snapshot('after')
  wundo('bytes')
end)

-- 18. An empty buffer: `UH_EMPTYBUF` in `uh_flags`.
case('emptybuf', function()
  fresh('emptybuf', { 'one', 'two' })
  run('delete', '%delete')
  sync()
  vim.api.nvim_buf_set_lines(0, 0, -1, true, { 'refilled' })
  sync()
  run('delete', '%delete')
  sync()
  snapshot('after')
  wundo('emptybuf')
end)

-- 19. The 'undofile' auto-write path: `u_write_undo` with `name == NULL`,
--     which is what derives the name from 'undodir'.  Also the one place
--     `undofile()` is exercised.
case('autowrite', function()
  local file = fresh('autowrite', { 'auto' })
  vim.o.undofile = true
  for i = 1, 3 do
    vim.api.nvim_buf_set_lines(0, -1, -1, true, { ('auto %d'):format(i) })
    sync()
  end
  run('write', 'write')
  emit('  undofile() |', scrub(vim.fn.undofile(file)))
  snapshot('after')
  index = index + 1
  keep(('%02d-autowrite.un~'):format(index), vim.fn.undofile(file))
end)

-- 20. The round trip: write, throw the buffer away, read it back with
--     `:rundo`, and check the tree survived -- then write it again and
--     let the hashes decide whether the second file matches the first.
case('roundtrip', function()
  local file = fresh('roundtrip', { 'rt one', 'rt two' })
  for i = 1, 4 do
    vim.api.nvim_buf_set_lines(0, -1, -1, true, { ('rt %d'):format(i) })
    sync()
  end
  run('undo', 'undo | undo')
  vim.api.nvim_buf_set_lines(0, -1, -1, true, { 'rt alt' })
  sync()
  run('write', 'silent write')
  snapshot('before')
  local first = wundo('roundtrip-a')

  -- A fresh buffer over the same file, with no tree of its own.
  vim.cmd('silent! %bwipeout!')
  vim.cmd('edit! ' .. vim.fn.fnameescape(file))
  setul(1000)
  snapshot('reloaded')
  run('rundo', 'rundo ' .. vim.fn.fnameescape(first))
  snapshot('afterrundo')

  -- Written straight back out, with nothing walked in between: read
  -- then write has to be the identity, so this one's scrubbed bytes
  -- must hash the same as roundtrip-a's.  The hashes artifact ends with
  -- the identical-hash groups, which is where that shows.
  wundo('roundtrip-b')

  -- And now the same tree after a walk.  This one is deliberately *not*
  -- expected to match: `u_undoredo` resolves an entry's `ue_bot` from
  -- the sentinel 0 ("to the end of the buffer") to a real line number as
  -- it applies it, so walking the tree rewrites a field the file
  -- carries.
  keys('g-')
  emit('  after g- |', esc(table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, true), '\\n')))
  keys('g+')
  emit('  after g+ |', esc(table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, true), '\\n')))
  wundo('roundtrip-c')
end)

emit('')
emit(('kept %d artifacts'):format(kept))
