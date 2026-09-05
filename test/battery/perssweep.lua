-- Driver for the persistence differential sweep; see perssweep.sh.
--
-- Covers the four on-disk surfaces of batch B10:
--
--   swap-write    memline.rs + memfile.rs -- the block tree a session
--                 leaves in its .swp, forced out with `:preserve`
--   swap-read     `:recover`, including every damaged-file rejection
--                 the reader has a message for, plus the SwapExists
--                 detection path and its six v:swapchoice answers
--   shada         shada.rs -- write, no-merge write, merge over a fixed
--                 seed, read back, and the damaged-file rejections
--   fileio        fileio.rs + bufwrite.rs -- what `:w` puts on disk
--                 under the write options, and what `:e` makes of the
--                 line endings, BOMs and encodings it is handed
--
-- Everything printed has to be reproducible across two builds run
-- minutes apart, so the report never carries a path outside the work
-- directory, a duration, a pid or a wall-clock time.  Produced files are
-- copied into $SWEEP_ART for the shell wrapper to scrub and hash; the
-- report is the readable half and the hashes are the byte-exact half.

local work = assert(os.getenv('SWEEP_WORK'), 'SWEEP_WORK unset')
local art = assert(os.getenv('SWEEP_ART'), 'SWEEP_ART unset')
local seeds = work .. '/seeds'

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
  -- Explicitly, not on collection: nvim reads several of these back in
  -- the same process and an unflushed copy reads as a truncated file.
  fd:close()
end

--- Escape to one printable line, so a byte difference shows in the diff.
local function esc(bytes)
  return (bytes:gsub('[^\32-\126]', function(c)
    return string.format('\\x%02x', c:byte())
  end):gsub('\\', '\\'))
end

--- Escaped in full when short, otherwise a prefix plus a digest: a
--- 70,000 byte line would otherwise be six copies of itself in the
--- report, and the digest catches a difference anywhere in it.
local function brief(bytes)
  if #bytes <= 200 then
    return esc(bytes)
  end
  return ('%s... %d bytes sha %s'):format(esc(bytes:sub(1, 100)), #bytes, vim.fn.sha256(bytes))
end

local runtime = os.getenv('VIMRUNTIME') or ''

--- Strip the bits of a message that name where the run happened.
local function scrub(text)
  text = tostring(text)
  text = text:gsub(vim.pesc(work), '<WORK>')
  -- Messages are truncated to the screen width, which lops the leading
  -- characters off a long path; the tail still has to be recognised.
  text = text:gsub(vim.pesc(work:sub(2)), '<WORK>')
  if runtime ~= '' then
    text = text:gsub(vim.pesc(runtime), '<RUNTIME>')
  end
  -- ml_recover and the ATTENTION message both print the pid stored in
  -- block zero and the swap file's date; neither is a function of what
  -- the editor did.  That the field was *reported at all* still is, so
  -- the labels stay and only the values go.
  -- Block zero carries the host and user name that wrote it, and both
  -- the ATTENTION message and the byte-order complaint print them back.
  local host = vim.uv.os_gethostname()
  if host and host ~= '' then
    text = text:gsub(vim.pesc(host), '<HOST>')
  end
  local user = (vim.uv.os_get_passwd() or {}).username
  if user and user ~= '' then
    text = text:gsub(vim.pesc(user), '<USER>')
  end
  text = text:gsub('(STILL RUNNING: )%d+', '%1<PID>')
  text = text:gsub('(process ID: )%d+', '%1<PID>')
  text = text:gsub('(from Nvim process )%d+', '%1<PID>')
  text = text:gsub('(dated: )[^\n]*', '%1<DATE>')
  -- Message text picks up trailing blanks depending on cursor position.
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
    emit(label, '|', (out:gsub('\n', '\n' .. label .. ' | ')))
  else
    emit(label, '| (silent)')
  end
end

local function mkdir(path)
  vim.fn.mkdir(path, 'p')
  return path
end

--- Copy a produced file into the artifact tree under a flat name.
local function keep(name, path)
  local bytes = slurp(path)
  if not bytes then
    emit('  keep', name, 'MISSING')
    return false
  end
  spew(art .. '/' .. name, bytes)
  emit('  keep', name, #bytes, 'bytes')
  return true
end

--- Reset to a clean, file-less state between cases.
local function reset()
  exec('silent! %bwipeout!')
  exec('enew!')
  vim.o.swapfile = true
  -- `nvim -l` sets no_swap_file, which zeroes 'updatecount', which is
  -- what ml_open checks before creating a swap file at all.  Putting it
  -- back is what makes the swap half of this sweep observable; the value
  -- is high enough that no automatic sync happens between `:preserve`
  -- calls, so what lands on disk stays a function of the edits.
  vim.o.updatecount = 200
  vim.o.backup = false
  vim.o.writebackup = true
  vim.o.patchmode = ''
  vim.o.backupcopy = 'auto'
  vim.o.backupext = '~'
  -- The default 'backupskip' contains "$TMPDIR/*", and this sweep's work
  -- directory lives under /tmp, so leaving it alone would silently turn
  -- every backup and patchmode case into a no-op.  One case below puts
  -- the default back on purpose, to keep the skip itself covered.
  vim.o.backupskip = ''
  vim.o.binary = false
  vim.o.bomb = false
  vim.o.endofline = true
  vim.o.fixendofline = true
  vim.o.fileformat = 'unix'
  vim.o.fileformats = 'unix,dos'
  vim.o.fileencoding = ''
  vim.o.fileencodings = 'ucs-bom,utf-8,default,latin1'
end

-- The ATTENTION dialog cannot be answered in a headless process, and the
-- documented way past it is the autocommand.  Every case that opens a
-- file with a swap present sets `choice` first; 'e' (edit anyway) is the
-- default so an unexpected detection cannot hang the sweep.
local swapchoice = 'e'
local swaplog = {}
vim.api.nvim_create_autocmd('SwapExists', {
  callback = function()
    swaplog[#swaplog + 1] = scrub(vim.v.swapname)
    vim.v.swapchoice = swapchoice
  end,
})

local function takeswaplog()
  local out = table.concat(swaplog, ' ')
  swaplog = {}
  return out == '' and '(none)' or out
end

-- =========================================================================
-- 1. swap write -- what a session leaves in its .swp
-- =========================================================================

local function repeated(n, make)
  local out = {}
  for i = 1, n do
    out[i] = make(i)
  end
  return out
end

-- Each case gets its own directory: swap file names are derived from the
-- edited file's name, and a collision would silently rename the second
-- one to .swo, making case N depend on case N-1.
local SWAP_CASES = {
  {
    'small',
    function(buf)
      vim.api.nvim_buf_set_lines(buf, 0, -1, true, { 'alpha', 'beta', 'gamma' })
    end,
  },
  {
    'empty',
    function() end,
  },
  {
    -- Enough lines to fill one data block and force a second, so the
    -- pointer block gets a second entry.
    'manylines',
    function(buf)
      vim.api.nvim_buf_set_lines(
        buf,
        0,
        -1,
        true,
        repeated(500, function(i)
          return ('line %d of the file, with some filler text'):format(i)
        end)
      )
    end,
  },
  {
    -- Deep enough that the pointer block itself splits and a second
    -- level appears above it.
    'deep',
    function(buf)
      vim.api.nvim_buf_set_lines(
        buf,
        0,
        -1,
        true,
        repeated(20000, function(i)
          return ('%d'):format(i)
        end)
      )
    end,
  },
  {
    -- Deep enough that the ROOT pointer block splits and a *second level*
    -- of pointer blocks appears under it.  `deep` above never gets there:
    -- 20000 short lines pack ~450 to a data block, so 48 entries sit in a
    -- root that holds 170.  Here each line is a whole 1000 bytes, four to
    -- a 4096-byte data block, so 700 lines need 175 data blocks and the
    -- root has to grow a level.  The middle insert and the delete then
    -- drive ml_add_stack/ml_lineadd/ml_find_line two levels down, which
    -- is the only thing that tells a one-level tree from a two-level one.
    'twolevel',
    function(buf)
      vim.api.nvim_buf_set_lines(
        buf,
        0,
        -1,
        true,
        repeated(700, function(i)
          return ('%06d '):format(i) .. string.rep('x', 993)
        end)
      )
      -- Split a data block hanging off the second-level pointer block.
      for i = 1, 20 do
        vim.api.nvim_buf_set_lines(
          buf,
          350,
          350,
          true,
          { ('ins %03d '):format(i) .. string.rep('y', 900) }
        )
      end
      -- And empty enough blocks that pointer entries are removed two
      -- levels down.
      vim.api.nvim_buf_set_lines(buf, 100, 220, true, {})
    end,
  },
  {
    -- One line longer than a page: ml_append allocates a multi-page data
    -- block for it.
    'longline',
    function(buf)
      vim.api.nvim_buf_set_lines(buf, 0, -1, true, {
        'before',
        string.rep('x', 20000),
        'after',
      })
    end,
  },
  {
    -- Inserting in the middle repeatedly is what drives ml_append_int's
    -- block-split branch rather than its append-at-end one.
    'splits',
    function(buf)
      vim.api.nvim_buf_set_lines(buf, 0, -1, true, repeated(200, function(i)
        return ('base %d'):format(i)
      end))
      for i = 1, 100 do
        vim.api.nvim_buf_set_lines(buf, 50, 50, true, { ('inserted %d'):format(i) })
      end
    end,
  },
  {
    -- Deleting most of a large buffer leaves free blocks behind, which
    -- is the only way to reach mf_free_bhdr's reuse path.
    'deletes',
    function(buf)
      vim.api.nvim_buf_set_lines(buf, 0, -1, true, repeated(600, function(i)
        return ('victim %d with padding to make the block fill up'):format(i)
      end))
      vim.api.nvim_buf_set_lines(buf, 100, 500, true, {})
    end,
  },
  {
    'unicode',
    function(buf)
      vim.api.nvim_buf_set_lines(buf, 0, -1, true, {
        'ascii',
        'naïve café',
        'κόσμε',
        '日本語のテキスト',
        '𝔘𝔫𝔦𝔠𝔬𝔡𝔢',
      })
    end,
  },
  {
    -- A NUL in a buffer line is stored as a NL byte in the block; the
    -- swap is where that substitution is visible.
    'nulbytes',
    function(buf)
      vim.api.nvim_buf_set_lines(buf, 0, -1, true, { 'a\0b', 'plain', '\0\0\0' })
    end,
  },
  {
    'ff_dos',
    function(buf)
      vim.bo[buf].fileformat = 'dos'
      vim.api.nvim_buf_set_lines(buf, 0, -1, true, { 'one', 'two' })
    end,
  },
  {
    'ff_mac',
    function(buf)
      vim.bo[buf].fileformat = 'mac'
      vim.api.nvim_buf_set_lines(buf, 0, -1, true, { 'one', 'two' })
    end,
  },
  {
    -- B0_HAS_FENC: the encoding name is tucked into the tail of
    -- b0_fname, which is the one variable-length field in block zero.
    'fenc_latin1',
    function(buf)
      vim.bo[buf].fileencoding = 'latin1'
      vim.api.nvim_buf_set_lines(buf, 0, -1, true, { 'latin' })
    end,
  },
  {
    'fenc_long',
    function(buf)
      vim.bo[buf].fileencoding = 'iso-8859-15'
      vim.bo[buf].bomb = true
      vim.api.nvim_buf_set_lines(buf, 0, -1, true, { 'bom' })
    end,
  },
}

local SETUPS = {}
for _, case in ipairs(SWAP_CASES) do
  SETUPS[case[1]] = case[2]
end

--- Leave behind the file/swap pair a crashed session would have left.
---
--- The swap is deleted when the buffer is wiped, so it is copied out
--- first and put back afterwards.  Planting it this way rather than
--- copying one in from elsewhere is what makes block zero *about* the
--- file it sits next to: `findswapname` ignores a swap whose `b0_fname`
--- names a different path (`differ` -> no ATTENTION), and `ml_recover`
--- warns E308 unless `b0_mtime` equals the file's mtime -- which, for a
--- copied-in pair, depends on whether the two runs happened to land in
--- the same wall-clock second.
local function plant(dir, base, setup, label)
  reset()
  -- 'directory' with a trailing "//" would encode the full path into the
  -- swap file's *name*; a plain directory keeps the name short and the
  -- B0_SAME_DIR flag meaningful.
  vim.o.directory = dir
  local path = dir .. '/' .. base
  run(label, 'edit! ' .. vim.fn.fnameescape(path))
  setup(vim.api.nvim_get_current_buf())
  run(label, 'write')
  -- Written and then dirtied, so b0_dirty is set for every case: an
  -- unmodified buffer's swap has nothing interesting past block zero,
  -- and `swapfile_unchanged` would have the detector delete it silently.
  vim.api.nvim_buf_set_lines(0, 0, 0, true, { 'DIRTY' })
  run(label, 'preserve')
  emit(label, 'lines', vim.api.nvim_buf_line_count(0))
  local swapname = vim.fn.swapname('%')
  emit(label, 'swapname', scrub(swapname))
  local bytes = slurp(swapname) or ''
  exec('silent! %bwipeout!')
  spew(swapname, bytes)
  return swapname, bytes, path
end

local function swap_write()
  emit('======== swap-write')
  for _, case in ipairs(SWAP_CASES) do
    local name, setup = case[1], case[2]
    local dir = mkdir(work .. '/swap/' .. name)
    local swapname = plant(dir, name .. '.txt', setup, name)
    keep('swap-' .. name .. '.swp', swapname)
  end

  -- 'swapfile' off: nothing should appear at all.
  reset()
  local dir = mkdir(work .. '/swap/noswap')
  vim.o.directory = dir
  vim.bo.swapfile = false
  run('noswap', 'edit ' .. vim.fn.fnameescape(dir .. '/noswap.txt'))
  vim.api.nvim_buf_set_lines(0, 0, -1, true, { 'no swap here' })
  run('noswap', 'write')
  run('noswap', 'preserve')
  emit('noswap', 'swapname', scrub(vim.fn.swapname('%')))

  -- An unnamed buffer never gets one either, and `:preserve` has a
  -- dedicated error for it (E313).
  reset()
  vim.bo.swapfile = false
  vim.api.nvim_buf_set_lines(0, 0, -1, true, { 'scratch' })
  run('unnamed', 'preserve')
end

-- =========================================================================
-- 2. swap read -- :recover, damaged swaps, and the detection path
-- =========================================================================

-- Byte edits against a known-good swap, each aimed at one rejection in
-- ml_recover.  Offsets are block-zero field offsets, which the format
-- fixes; see swapscrub.py for the layout.
local SWAP_DAMAGE = {
  { 'bad_id', { [1] = 0x58 } },
  { 'old_version', 'version', 'VIM 3.0\0\0\0' },
  { 'bad_magic_long', { [1009] = 0xFF } },
  { 'bad_magic_char', { [1023] = 0x00 } },
  { 'bad_page_size', { [13] = 0x00, [14] = 0x02, [15] = 0, [16] = 0 } },
  { 'block1_id', 'page1', 'zz' },
  { 'data_id', 'page2', 'zz' },
  { 'truncated', 'truncate', 2048 },
  { 'empty', 'truncate', 0 },
}

local function patch(bytes, edits)
  for at, byte in pairs(edits) do
    bytes = bytes:sub(1, at - 1) .. string.char(byte) .. bytes:sub(at + 1)
  end
  return bytes
end

local function swap_read()
  emit('======== swap-recover')
  -- The undamaged pairs first: `:recover` has to reproduce the buffer
  -- that was preserved, including the unwritten DIRTY line.
  for _, name in
    ipairs({ 'small', 'manylines', 'twolevel', 'longline', 'splits', 'deletes', 'nulbytes', 'unicode', 'ff_dos' })
  do
    local dir = mkdir(work .. '/recover/' .. name)
    plant(dir, name .. '.txt', SETUPS[name], 'plant-' .. name)
    reset()
    vim.o.directory = dir
    swapchoice = 'e'
    run('rec-' .. name, 'recover ' .. vim.fn.fnameescape(dir .. '/' .. name .. '.txt'))
    local lines = vim.api.nvim_buf_get_lines(0, 0, -1, true)
    emit('rec-' .. name, 'lines', #lines, 'modified', tostring(vim.bo.modified))
    emit('rec-' .. name, 'first', esc(lines[1] or ''))
    emit('rec-' .. name, 'last', esc(lines[#lines] or ''))
    -- A hash over the whole recovered buffer, so a difference anywhere
    -- in a 20k-line case is caught without printing 20k lines.
    emit('rec-' .. name, 'sha', vim.fn.sha256(table.concat(lines, '\n')))
    emit('rec-' .. name, 'ff', vim.bo.fileformat, 'fenc', vim.bo.fileencoding)
    emit('rec-' .. name, 'swapexists', takeswaplog())
  end

  emit('======== swap-damage')
  for _, damage in ipairs(SWAP_DAMAGE) do
    local label, how, arg = damage[1], damage[2], damage[3]
    local dir = mkdir(work .. '/damage/' .. label)
    -- Damaged per case rather than once: the good pair has to sit in the
    -- directory it names, and each case rewrites the swap in place.
    local swapname, good = plant(dir, 'dmg.txt', SETUPS.manylines, 'plant-' .. label)
    local bytes = good
    if type(how) == 'table' then
      bytes = patch(bytes, how)
    elseif how == 'truncate' then
      bytes = bytes:sub(1, arg)
    elseif how == 'version' then
      bytes = bytes:sub(1, 2) .. arg .. bytes:sub(13)
    elseif how == 'page1' then
      bytes = bytes:sub(1, 4096) .. arg .. bytes:sub(4099)
    elseif how == 'page2' then
      bytes = bytes:sub(1, 8192) .. arg .. bytes:sub(8195)
    end
    spew(swapname, bytes)
    reset()
    vim.o.directory = dir
    swapchoice = 'e'
    run('dmg-' .. label, 'recover ' .. vim.fn.fnameescape(dir .. '/dmg.txt'))
    emit('dmg-' .. label, 'lines', vim.api.nvim_buf_line_count(0))
    emit(
      'dmg-' .. label,
      'first',
      esc((vim.api.nvim_buf_get_lines(0, 0, 1, false))[1] or '')
    )
    emit('dmg-' .. label, 'swapexists', takeswaplog())
  end

  emit('======== swap-detect')
  -- Opening a file that already has a swap, once per v:swapchoice.  The
  -- planted swap carries this process's own pid, so the detector takes
  -- its "STILL RUNNING" branch -- the value is masked, the branch is not.
  -- Two flavours per choice.  `live` leaves this process's pid in block
  -- zero, which `swapfile_proc_running` recognises and short-circuits
  -- (W325).  `dead` zeroes it, which is the "no pid recorded" case and
  -- the only way to reach the full ATTENTION path from a test.
  local DETECT = {}
  for _, choice in ipairs({ 'e', 'o', 'r', 'q', 'a', 'd' }) do
    DETECT[#DETECT + 1] = { choice, 'dead' }
  end
  DETECT[#DETECT + 1] = { 'e', 'live' }
  for _, spec in ipairs(DETECT) do
    local choice, flavour = spec[1], spec[2] .. '-' .. spec[1]
    local dir = mkdir(work .. '/detect/' .. flavour)
    local swapname, bytes = plant(dir, 'det.txt', SETUPS.small, 'plant-det-' .. flavour)
    if spec[2] == 'dead' then
      spew(swapname, bytes:sub(1, 24) .. '\0\0\0\0' .. bytes:sub(29))
    end
    reset()
    vim.o.directory = dir
    swapchoice = choice
    run('det-' .. flavour, 'edit ' .. vim.fn.fnameescape(dir .. '/det.txt'))
    emit('det-' .. flavour, 'swapexists', takeswaplog())
    emit(
      'det-' .. flavour,
      'buf',
      scrub(vim.api.nvim_buf_get_name(0)),
      'ro',
      tostring(vim.bo.readonly),
      'lines',
      vim.api.nvim_buf_line_count(0)
    )
    emit('det-' .. flavour, 'swapleft', tostring(vim.uv.fs_stat(swapname) ~= nil))
  end

  emit('======== swap-inspect')
  -- `:swapname`, and the swapfile-name search that picks .swo/.swn when
  -- the obvious name is taken.
  local dir = mkdir(work .. '/names')
  local swapname, bytes = plant(dir, 'n.txt', SETUPS.small, 'plant-names')
  -- The .swo copy is a *different* file's swap as far as block zero is
  -- concerned, which is exactly the case that has findswapname keep
  -- walking down the extension rather than stop and complain.
  spew(swapname:gsub('p$', 'o'), bytes)
  reset()
  vim.o.directory = dir
  swapchoice = 'e'
  run('names', 'edit ' .. vim.fn.fnameescape(dir .. '/n.txt'))
  run('names', 'swapname')
  emit('names', 'swapexists', takeswaplog())

  -- swapinfo() reads block zero without loading the file: a second,
  -- independent decoder of the same struct.
  local info = vim.fn.swapinfo(swapname)
  local keys = {}
  for k in pairs(info) do
    keys[#keys + 1] = k
  end
  table.sort(keys)
  for _, k in ipairs(keys) do
    -- pid, mtime, user and host are machine and clock facts; that the
    -- key is present is not.
    -- inode is a filesystem fact, like pid/mtime/user/host.
    local masked = k == 'pid' or k == 'mtime' or k == 'user' or k == 'host' or k == 'inode'
    emit(
      'swapinfo',
      k,
      masked and ('<SET:' .. tostring(info[k] ~= nil) .. '>') or scrub(tostring(info[k]))
    )
  end
  emit('swapinfo', 'swapfilelist', scrub(table.concat(vim.fn.swapfilelist(), ' ')))
end

-- =========================================================================
-- 2b. byte offsets -- ml_updatechunk / ml_find_line_or_offset
-- =========================================================================

-- These two are reachable only through line2byte(), byte2line() and the
-- word-count machinery, and nothing in the tree drives them against a
-- buffer large enough to hold more than one chunk (ml_updatechunk keeps
-- a chunk index that only exists once a buffer is long enough).  Without
-- this section a rewrite of the chunk index would pass every suite.
local function byte_offsets()
  emit('======== byte-offsets')
  for _, ff in ipairs({ 'unix', 'dos', 'mac' }) do
    reset()
    vim.bo.fileformat = ff
    vim.api.nvim_buf_set_lines(
      0,
      0,
      -1,
      true,
      repeated(5000, function(i)
        return string.rep('x', i % 37) .. (' %d'):format(i)
      end)
    )
    local label = 'off-' .. ff
    local acc = {}
    for _, lnum in ipairs({ 1, 2, 799, 800, 801, 1000, 2500, 4999, 5000, 5001, -1 }) do
      acc[#acc + 1] = ('%d=%d'):format(lnum, vim.fn.line2byte(lnum))
    end
    emit(label, 'line2byte', table.concat(acc, ' '))
    acc = {}
    for _, off in ipairs({ 1, 2, 100, 5000, 50000, 100000, 1000000, -1 }) do
      acc[#acc + 1] = ('%d=%d'):format(off, vim.fn.byte2line(off))
    end
    emit(label, 'byte2line', table.concat(acc, ' '))
    emit(label, 'wordcount', (scrub(vim.inspect(vim.fn.wordcount())):gsub('%s+', ' ')))

    -- The chunk index is maintained incrementally, so what it reports
    -- after edits is a different question from what it reports after a
    -- load.
    vim.api.nvim_buf_set_lines(0, 100, 900, true, {})
    vim.api.nvim_buf_set_lines(
      0,
      50,
      50,
      true,
      repeated(300, function(i)
        return ('inserted %d'):format(i)
      end)
    )
    vim.api.nvim_buf_set_lines(0, 3000, 3001, true, { string.rep('y', 4000) })
    acc = {}
    for _, lnum in ipairs({ 1, 50, 351, 352, 1000, 3000, 3001, 4500, 4501 }) do
      acc[#acc + 1] = ('%d=%d'):format(lnum, vim.fn.line2byte(lnum))
    end
    emit(label, 'line2byte-after-edits', table.concat(acc, ' '))
    acc = {}
    for _, off in ipairs({ 1, 1000, 20000, 60000, 120000 }) do
      acc[#acc + 1] = ('%d=%d'):format(off, vim.fn.byte2line(off))
    end
    emit(label, 'byte2line-after-edits', table.concat(acc, ' '))
    emit(label, 'wordcount-after-edits', (scrub(vim.inspect(vim.fn.wordcount())):gsub('%s+', ' ')))
  end
end

-- =========================================================================
-- 3. shada -- write, merge, read, reject
-- =========================================================================

--- Populate the state shada is supposed to persist.
local function shada_state(tag)
  local dir = mkdir(work .. '/shada')
  local f1 = dir .. '/' .. tag .. '-one.txt'
  local f2 = dir .. '/' .. tag .. '-two.txt'
  spew(f1, 'first file line one\nfirst file line two\nfirst file line three\n')
  spew(f2, 'second file\nwith two lines\n')

  exec('edit ' .. vim.fn.fnameescape(f1))
  vim.api.nvim_win_set_cursor(0, { 2, 3 })
  exec('normal! majmb')
  exec('mark A')
  exec('edit ' .. vim.fn.fnameescape(f2))
  vim.api.nvim_win_set_cursor(0, { 2, 1 })
  exec('mark B')
  exec("normal! ochanged\27")
  exec('normal! u')

  -- Registers, one per type plus the numbered ones.
  vim.fn.setreg('a', 'charwise contents', 'c')
  vim.fn.setreg('b', { 'linewise one', 'linewise two' }, 'l')
  vim.fn.setreg('c', { 'blk1', 'blk2' }, 'b12')
  vim.fn.setreg('"', 'unnamed contents', 'c')
  vim.fn.setreg('1', 'numbered one', 'l')
  vim.fn.setreg('2', 'numbered two', 'l')
  -- A NUL cannot travel through vim.fn.setreg (a Lua string holding one
  -- arrives as a Blob), so the register that carries awkward bytes is
  -- built by yanking a buffer line instead -- which is how a NUL gets
  -- into a register in real use anyway.
  vim.api.nvim_buf_set_lines(0, 0, 0, true, { 'a\0b\tc\rd' })
  exec('normal! ggv$"zy')
  exec('normal! ggdd')

  -- History, one entry per type that shada records.
  vim.fn.histadd('cmd', 'echo "history cmd"')
  vim.fn.histadd('cmd', 'set list')
  vim.fn.histadd('search', 'searchpattern')
  vim.fn.histadd('expr', '1 + 2')
  vim.fn.histadd('input', 'typed input')

  -- Variables: shada only keeps ALLCAPS globals, and only the types
  -- encode_vim_to_msgpack has a representation for.
  vim.g.SWEEPSTR = 'string value'
  vim.g.SWEEPNUM = 42
  vim.g.SWEEPFLOAT = 1.5
  vim.g.SWEEPLIST = { 1, 'two', { 3 } }
  vim.g.SWEEPDICT = { a = 1, b = 'two' }
  vim.g.SWEEPBLOB = vim.fn.eval('0z00112233')
  vim.g.sweeplower = 'not written'

  -- Search and substitute patterns.
  exec('silent! normal! /line\r')
  exec('silent! %s/nothing-matches-this/replacement/e')
end

local function shada_dump(label, path)
  if not keep('shada-' .. label .. '.shada', path) then
    return
  end
end

local function shada_read_state(label)
  emit(label, 'reg-a', esc(vim.fn.getreg('a')), vim.fn.getregtype('a'))
  emit(label, 'reg-b', esc(vim.fn.getreg('b')), vim.fn.getregtype('b'))
  emit(label, 'reg-c', esc(vim.fn.getreg('c')), vim.fn.getregtype('c'))
  emit(label, 'reg-quote', esc(vim.fn.getreg('"')), vim.fn.getregtype('"'))
  emit(label, 'reg-z', esc(vim.fn.getreg('z')))
  for _, h in ipairs({ 'cmd', 'search', 'expr', 'input' }) do
    local items = {}
    for i = 1, vim.fn.histnr(h) do
      items[#items + 1] = vim.fn.histget(h, i)
    end
    emit(label, 'hist-' .. h, esc(table.concat(items, '|')))
  end
  for _, name in ipairs({ 'SWEEPSTR', 'SWEEPNUM', 'SWEEPFLOAT', 'SWEEPLIST', 'SWEEPDICT', 'SWEEPBLOB', 'SEEDVAR', 'sweeplower' }) do
    local ok, v = pcall(function()
      return vim.api.nvim_get_var(name)
    end)
    emit(label, 'var-' .. name, ok and scrub(vim.inspect(v)):gsub('%s+', ' ') or 'unset')
  end
  for _, m in ipairs({ 'A', 'B', 'C' }) do
    emit(label, 'mark-' .. m, scrub(vim.inspect(vim.fn.getpos("'" .. m))))
  end
  emit(label, 'oldfiles', scrub(table.concat(vim.v.oldfiles, ' ')))
  emit(label, 'search', esc(vim.fn.getreg('/')))
end

local SHADA_DAMAGE = {
  { 'truncated_header', 6 },
  { 'truncated_payload', 40 },
  { 'garbage', nil, string.rep('\255', 64) },
  { 'zero', nil, string.rep('\0', 16) },
  -- A well-formed entry header whose declared length runs past the end.
  { 'long_length', nil, '\5\206\58\221\179\0\206\0\0\255\255' },
  -- Entry type past the last known one: skipped, not an error.
  { 'unknown_type', nil, '\100\206\58\221\179\0\1\192' },
  -- A register entry whose payload is an integer instead of a map.
  { 'wrong_payload', nil, '\5\206\58\221\179\0\1\42' },
}

local function shada()
  emit('======== shada-write')
  reset()
  vim.o.shada = "!,'100,<50,s10,h"
  shada_state('w')
  local dir = mkdir(work .. '/shada')

  run('sh-nomerge', 'wshada! ' .. vim.fn.fnameescape(dir .. '/nomerge.shada'))
  shada_dump('nomerge', dir .. '/nomerge.shada')

  -- Merge: the same state written over a fixed seed produced by the
  -- packer, so what merging does is a function of two known inputs.
  spew(dir .. '/merge.shada', slurp(seeds .. '/seed.shada') or '')
  run('sh-merge', 'wshada ' .. vim.fn.fnameescape(dir .. '/merge.shada'))
  shada_dump('merge', dir .. '/merge.shada')

  -- 'shada' controls which entry types are written and how many of each.
  local OPTS = {
    { 'minimal', "'0,<0,s0,h" },
    { 'nohist', "'50,<10,s5" },
    { 'buflist', "'50,%10,<10,s5,h" },
    { 'bigs', "'100,<1000,s100,h" },
    { 'noquote', "'100,\"0,s10,h" },
  }
  for _, opt in ipairs(OPTS) do
    local name, value = opt[1], opt[2]
    run('sh-opt-' .. name, 'set shada=' .. vim.fn.escape(value, ' \\|"'))
    run(
      'sh-opt-' .. name,
      'wshada! ' .. vim.fn.fnameescape(dir .. '/opt-' .. name .. '.shada')
    )
    shada_dump('opt-' .. name, dir .. '/opt-' .. name .. '.shada')
  end
  vim.o.shada = "!,'100,<50,s10,h"

  emit('======== shada-read')
  -- A fresh state: what comes back has to come from the file.
  reset()
  for _, name in ipairs({ 'a', 'b', 'c', 'z', '"', '1', '2' }) do
    vim.fn.setreg(name, '')
  end
  exec('silent! call histdel("cmd") | call histdel("search")')
  run('sh-rd-seed', 'rshada! ' .. vim.fn.fnameescape(seeds .. '/seed.shada'))
  shada_read_state('sh-rd-seed')

  run('sh-rd-own', 'rshada ' .. vim.fn.fnameescape(dir .. '/nomerge.shada'))
  shada_read_state('sh-rd-own')

  run('sh-rd-merged', 'rshada! ' .. vim.fn.fnameescape(dir .. '/merge.shada'))
  shada_read_state('sh-rd-merged')

  emit('======== shada-damage')
  local good = slurp(dir .. '/nomerge.shada') or ''
  for _, damage in ipairs(SHADA_DAMAGE) do
    local label, cut, raw = damage[1], damage[2], damage[3]
    local bytes = raw or good:sub(1, cut)
    local path = dir .. '/dmg-' .. label .. '.shada'
    spew(path, bytes)
    run('sh-dmg-' .. label, 'rshada ' .. vim.fn.fnameescape(path))
    -- Writing over a damaged file is the other half: the merge reader
    -- has to give up without destroying what it could not parse.
    run('sh-dmg-' .. label .. '-w', 'wshada ' .. vim.fn.fnameescape(path))
    emit('sh-dmg-' .. label, 'size', #(slurp(path) or ''))
  end

  emit('======== shada-missing')
  run('sh-missing-r', 'rshada ' .. vim.fn.fnameescape(dir .. '/does-not-exist.shada'))
  run('sh-missing-w', 'wshada ' .. vim.fn.fnameescape(dir .. '/fresh.shada'))
  shada_dump('fresh', dir .. '/fresh.shada')
end

-- =========================================================================
-- 4. fileio + bufwrite
-- =========================================================================

local READ_FIXTURES = {
  { 'unix', 'one\ntwo\nthree\n' },
  { 'dos', 'one\r\ntwo\r\nthree\r\n' },
  { 'mac', 'one\rtwo\rthree\r' },
  { 'mixed', 'one\r\ntwo\nthree\r\n' },
  { 'noeol', 'one\ntwo\nthree' },
  { 'empty', '' },
  { 'onlynl', '\n' },
  { 'nul', 'a\0b\nc\0\0d\n' },
  { 'bom_utf8', '\239\187\191one\ntwo\n' },
  { 'bom_utf16le', '\255\254o\0n\0e\0\10\0' },
  { 'bom_utf16be', '\254\255\0o\0n\0e\0\10' },
  { 'latin1', 'caf\233 na\239ve\n' },
  { 'longline', string.rep('q', 70000) .. '\n' },
  { 'crlf_noeol', 'one\r\ntwo\r' },
  { 'trailing_cr', 'one\ntwo\r\n' },
}

local function fileio_read()
  emit('======== fileio-read')
  local dir = mkdir(work .. '/read')
  for _, fx in ipairs(READ_FIXTURES) do
    local name, bytes = fx[1], fx[2]
    local path = dir .. '/' .. name .. '.in'
    spew(path, bytes)
    for _, ffs in ipairs({ 'unix,dos', 'dos,unix', 'unix', 'dos', 'mac,unix,dos', '' }) do
      reset()
      vim.o.fileformats = ffs
      local label = ('rd-%s[%s]'):format(name, ffs == '' and 'none' or ffs)
      run(label, 'edit! ' .. vim.fn.fnameescape(path))
      local lines = vim.api.nvim_buf_get_lines(0, 0, -1, true)
      emit(
        label,
        'n=' .. #lines,
        'ff=' .. vim.bo.fileformat,
        'fenc=' .. vim.bo.fileencoding,
        'bomb=' .. tostring(vim.bo.bomb),
        'eol=' .. tostring(vim.bo.endofline),
        'sha=' .. vim.fn.sha256(table.concat(lines, '\n'))
      )
      if #lines <= 4 then
        for i, line in ipairs(lines) do
          emit(label, i, brief(line))
        end
      end
    end
  end

  -- 'binary' and `:read` are separate entry points into the same reader.
  reset()
  vim.o.binary = true
  run('rd-binary', 'edit! ' .. vim.fn.fnameescape(dir .. '/dos.in'))
  emit(
    'rd-binary',
    esc(table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, true), '|')),
    'eol=' .. tostring(vim.bo.endofline)
  )
  reset()
  run('rd-into', 'enew!')
  run('rd-into', 'read ' .. vim.fn.fnameescape(dir .. '/unix.in'))
  emit('rd-into', esc(table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, true), '|')))
  run('rd-into', '0read ' .. vim.fn.fnameescape(dir .. '/mac.in'))
  emit('rd-into2', esc(table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, true), '|')))
  run('rd-missing', 'edit! ' .. vim.fn.fnameescape(dir .. '/absent.in'))
  run('rd-isdir', 'edit! ' .. vim.fn.fnameescape(dir))
end

local WRITE_CASES = {
  { 'plain', function() end },
  { 'ff_dos', function() vim.bo.fileformat = 'dos' end },
  { 'ff_mac', function() vim.bo.fileformat = 'mac' end },
  { 'noeol', function() vim.bo.endofline = false end },
  { 'binary_noeol', function() vim.bo.binary = true vim.bo.endofline = false end },
  { 'bom_utf8', function() vim.bo.bomb = true end },
  { 'fenc_latin1', function() vim.bo.fileencoding = 'latin1' end },
  { 'fenc_utf16', function() vim.bo.fileencoding = 'utf-16le' vim.bo.bomb = true end },
  { 'backup', function() vim.o.backup = true end },
  { 'backup_copy', function() vim.o.backup = true vim.o.backupcopy = 'yes' end },
  { 'backup_nocopy', function() vim.o.backup = true vim.o.backupcopy = 'no' end },
  { 'backupext', function() vim.o.backup = true vim.o.backupext = '.bak' end },
  { 'nowritebackup', function() vim.o.writebackup = false end },
  { 'patchmode', function() vim.o.patchmode = '.orig' end },
  { 'fsync', function() vim.o.fsync = true end },
  {
    'backup_skipped',
    function()
      vim.o.backup = true
      -- The work directory is under /tmp, which this pattern covers.
      vim.o.backupskip = '/tmp/*'
    end,
  },
}

local function listdir(dir)
  local names = {}
  for name in vim.fs.dir(dir) do
    names[#names + 1] = name
  end
  table.sort(names)
  return names
end

local function fileio_write()
  emit('======== fileio-write')
  local content = { 'first line', 'second líne', 'third\tline' }
  for _, case in ipairs(WRITE_CASES) do
    local name, setup = case[1], case[2]
    local dir = mkdir(work .. '/write/' .. name)
    reset()
    vim.o.directory = dir
    -- nvim's default 'backupdir' points at the XDG state directory, so
    -- without this the backup and patchmode cases would leave nothing in
    -- the case directory to compare -- and patchmode is implemented by
    -- renaming the backup, so it silently does nothing too.
    vim.o.backupdir = dir
    local path = dir .. '/out.txt'
    -- Pre-existing content, so the backup and patchmode branches have
    -- something to move aside.
    spew(path, 'original first\noriginal second\n')
    run('wr-' .. name, 'edit! ' .. vim.fn.fnameescape(path))
    setup()
    vim.api.nvim_buf_set_lines(0, 0, -1, true, content)
    run('wr-' .. name, 'write')
    -- Twice: patchmode and the backup rotation behave differently on a
    -- second write, and only the second one exercises the "backup
    -- already exists" branch.
    vim.api.nvim_buf_set_lines(0, 0, 0, true, { 'added later' })
    run('wr-' .. name, 'write')
    for _, entry in ipairs(listdir(dir)) do
      local bytes = slurp(dir .. '/' .. entry)
      if entry:match('%.sw[a-p]$') then
        -- The swap file is block-tree output and belongs to the swap
        -- half of the sweep; here only its presence matters -- and its
        -- bytes carry a pid and a host name, so they must not be
        -- inlined into the report.
        emit('wr-' .. name, 'file', entry, '(swap,', #(bytes or ''), 'bytes)')
      else
        emit('wr-' .. name, 'file', entry, #(bytes or ''), brief(bytes or ''))
      end
    end
  end

  emit('======== fileio-write-partial')
  local dir = mkdir(work .. '/write/partial')
  reset()
  vim.o.directory = dir
  vim.api.nvim_buf_set_lines(0, 0, -1, true, { 'l1', 'l2', 'l3', 'l4', 'l5' })
  run('wp-range', '2,4write ' .. vim.fn.fnameescape(dir .. '/range.txt'))
  run('wp-append', '1write >> ' .. vim.fn.fnameescape(dir .. '/range.txt'))
  -- Writing over an existing file needs the bang; the forced write goes
  -- to its own target so the range result above survives to be compared.
  run('wp-exists', 'write ' .. vim.fn.fnameescape(dir .. '/range.txt'))
  run('wp-force', 'write! ' .. vim.fn.fnameescape(dir .. '/force.txt'))
  run('wp-force2', 'write! ' .. vim.fn.fnameescape(dir .. '/force.txt'))
  for _, entry in ipairs(listdir(dir)) do
    if not entry:match('%.sw[a-p]$') then
      emit('wp', entry, esc(slurp(dir .. '/' .. entry) or ''))
    end
  end

  emit('======== fileio-write-errors')
  local ro = mkdir(work .. '/write/readonly')
  spew(ro .. '/ro.txt', 'read only\n')
  vim.uv.fs_chmod(ro .. '/ro.txt', 292) -- 0444
  reset()
  vim.o.directory = ro
  run('we-ro', 'edit! ' .. vim.fn.fnameescape(ro .. '/ro.txt'))
  emit('we-ro', 'readonly=' .. tostring(vim.bo.readonly))
  vim.api.nvim_buf_set_lines(0, 0, -1, true, { 'changed' })
  run('we-ro', 'write')
  run('we-ro-force', 'write!')
  emit('we-ro', 'after', esc(slurp(ro .. '/ro.txt') or ''))
  vim.uv.fs_chmod(ro .. '/ro.txt', 420) -- 0644
  run('we-dir', 'write! ' .. vim.fn.fnameescape(ro))
  run('we-nodir', 'write! ' .. vim.fn.fnameescape(ro .. '/missing/deep.txt'))

  emit('======== fileio-write-autocmd')
  -- BufWriteCmd replaces the writer wholesale; BufWritePre/Post bracket
  -- it.  Both reach bufwrite.rs's autocommand plumbing rather than its
  -- byte path, and the ordering is what a rewrite can lose.
  local dir2 = mkdir(work .. '/write/autocmd')
  reset()
  vim.o.directory = dir2
  local seen = {}
  for _, ev in ipairs({ 'BufWritePre', 'BufWritePost', 'FileWritePre', 'FileWritePost' }) do
    vim.api.nvim_create_autocmd(ev, {
      pattern = '*',
      callback = function()
        seen[#seen + 1] = ev
      end,
    })
  end
  vim.api.nvim_buf_set_lines(0, 0, -1, true, { 'auto one', 'auto two' })
  run('wa', 'write ' .. vim.fn.fnameescape(dir2 .. '/auto.txt'))
  run('wa', '1write ' .. vim.fn.fnameescape(dir2 .. '/auto-part.txt'))
  emit('wa', 'events', table.concat(seen, ','))
  exec('autocmd! BufWritePre')
  exec('autocmd! BufWritePost')
  exec('autocmd! FileWritePre')
  exec('autocmd! FileWritePost')
end

-- =========================================================================

swap_write()
swap_read()
byte_offsets()
shada()
fileio_read()
fileio_write()
emit('======== done')
