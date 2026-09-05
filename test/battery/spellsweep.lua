-- Driver for the spell differential sweep; see spellsweep.sh.
--
-- Compiles every corpus case with `:mkspell`, then loads the result back
-- and dumps everything observable about it.  Writes one canonical text
-- report to stdout; the .spl/.sug bytes are left on disk for the shell
-- wrapper to scrub and hash.
--
-- Everything printed has to be reproducible across two builds of nvim
-- run minutes apart, so the report never carries a path, a duration or
-- an address.  The corpus root is substituted out of message text.

local corpus = assert(os.getenv('SWEEP_CORPUS'), 'SWEEP_CORPUS unset')

local function read(path)
  local fd = io.open(path, 'r')
  if not fd then
    return nil
  end
  local text = fd:read('*a')
  fd:close()
  return text
end

local function lines(path)
  local text = read(path)
  if not text then
    return {}
  end
  local out = {}
  for line in text:gmatch('[^\n]+') do
    out[#out + 1] = line
  end
  return out
end

local function meta(dir)
  local out = {}
  for _, line in ipairs(lines(dir .. '/meta')) do
    local k, v = line:match('^(%w+)=(.*)$')
    if k then
      out[k] = v
    end
  end
  return out
end

--- Strip the bits of a message that name where the run happened.
local runtime = os.getenv('VIMRUNTIME') or ''

local function scrub(text)
  text = text:gsub(vim.pesc(corpus), '<CORPUS>')
  -- :spellinfo names the file it loaded, and the shipped dictionary
  -- lives under VIMRUNTIME -- which differs between a baseline worktree
  -- and the working tree.  That is where the binary came from, not what
  -- it did.
  if runtime ~= '' then
    text = text:gsub(vim.pesc(runtime), '<RUNTIME>')
  end
  -- Absolute paths leak through $HOME and the temp root as well.
  text = text:gsub('/[%w%./_%-]*/spellsweep%-[%w]+', '<WORK>')
  -- :mkspell's progress line is rate-limited to one a wall-clock second
  -- (`os_time() > last_msg_time` in spell_read_dic), so whether a given
  -- one appears depends on how fast the machine was that run.  Measured:
  -- the 24k-word case emitted two of them in one run of three and three
  -- in the others.  The word totals it would report are printed
  -- unconditionally further down, so nothing is lost by dropping it.
  text = text:gsub('line%s+%d+, word%s+%d+ %- [^\n]*\n?', '')
  -- Message text ends up with trailing blanks depending on where the
  -- cursor was; they are noise either way.
  text = text:gsub('%s+\n', '\n'):gsub('%s+$', '')
  return text
end

local function exec(cmd)
  local ok, res = pcall(vim.api.nvim_exec2, cmd, { output = true })
  if ok then
    return scrub(res.output or '')
  end
  -- Errors are part of the observable behaviour: a case that fails to
  -- compile has to fail the same way in both binaries.
  return 'ERROR ' .. scrub(tostring(res))
end

local function emit(...)
  io.write(table.concat({ ... }, ' '), '\n')
end

--- Render a Vim value with tables in a stable order.
local function show(v)
  if type(v) == 'table' then
    local parts = {}
    for _, item in ipairs(v) do
      parts[#parts + 1] = show(item)
    end
    return '[' .. table.concat(parts, ', ') .. ']'
  end
  return tostring(v)
end

local cases = {}
for name, kind in vim.fs.dir(corpus) do
  if kind == 'directory' then
    cases[#cases + 1] = name
  end
end
table.sort(cases)

-- 'spellsuggest' picks the scoring engine: `best` runs both the
-- edit-distance and the soundfold pass and merges, `double` scores each
-- separately, `fast` skips soundfolding entirely.  They are three
-- different code paths through spellsuggest.rs, so all three are swept.
local SUGGEST_MODES = { 'best', 'double', 'fast' }

for _, name in ipairs(cases) do
  local dir = corpus .. '/' .. name
  local m = meta(dir)
  local enc = m.enc == 'latin1' and 'latin1' or 'utf-8'
  emit('=== case', name, 'enc=' .. enc)

  vim.fn.mkdir(dir .. '/spell', 'p')

  -- --- write side -------------------------------------------------------
  local inputs = {}
  for word in (m.inputs or 't'):gmatch('%S+') do
    inputs[#inputs + 1] = word
  end
  local flag = m.ascii == '1' and '-ascii ' or ''
  -- The output is named after the case, not after its input.  Loaded
  -- spell files are cached by language name, so a shared name would
  -- make every case after the first read the first one's trie -- which
  -- is exactly what the first draft of this sweep did, reporting every
  -- word in every case as `bad` against `asciiflag`'s dictionary.
  local out = ('%s/spell/%s.%s.spl'):format(dir, name, enc)
  local cmd = ('mkspell! %s%s %s'):format(
    flag,
    vim.fn.fnameescape(out),
    table.concat(
      vim.tbl_map(function(i)
        return vim.fn.fnameescape(dir .. '/' .. i)
      end, inputs),
      ' '
    )
  )
  emit('--- mkspell')
  emit(exec(cmd))

  -- --- read side --------------------------------------------------------
  -- A fresh buffer per case: 'spell' state and the loaded language list
  -- are window-local, and a stale one would make a case depend on the
  -- one before it.
  exec('enew!')
  vim.wo.spell = true
  -- 'spelllang' takes a file name ending in .spl instead of a language,
  -- which sidesteps the runtimepath search and its name-based cache.
  -- It rejects `/` though (E474), so the name has to be relative and
  -- the window has to sit in the directory holding it.  Naming the file
  -- after the case is what keeps the cache from serving case N-1's trie.
  exec('lcd ' .. vim.fn.fnameescape(dir .. '/spell'))
  local set = exec(('setlocal spelllang=%s.%s.spl'):format(name, enc))
  if set ~= '' then
    emit('--- spelllang', set)
  end

  emit('--- spellinfo')
  emit(exec('spellinfo'))

  emit('--- verdicts')
  for _, word in ipairs(lines(dir .. '/words')) do
    local ok, res = pcall(vim.fn.spellbadword, word)
    emit(word, '->', ok and show(res) or ('ERROR ' .. scrub(tostring(res))))
  end

  emit('--- suggestions')
  for _, mode in ipairs(SUGGEST_MODES) do
    vim.o.spellsuggest = mode
    for _, word in ipairs(lines(dir .. '/sugs')) do
      for _, cap in ipairs({ 0, 1 }) do
        local ok, res = pcall(vim.fn.spellsuggest, word, 10, cap)
        emit(
          mode,
          'cap=' .. cap,
          word,
          '->',
          ok and show(res) or ('ERROR ' .. scrub(tostring(res)))
        )
      end
    end
  end
  vim.o.spellsuggest = 'best'
end

-- --- a .sug the reader has to reject -------------------------------------
-- suggest_load_files() has five ways out other than success and each one
-- is only reachable by damaging the file after :mkspell wrote it, so
-- nothing else in the sweep goes near them.  The damaged copies are made
-- from the `sal` case, which is small and has a .sug companion.
--
-- Every copy needs its own name: loaded spell files are cached by
-- language name, and reusing one would hand back the previous trie
-- instead of reading the damaged bytes.
local function slurp(path)
  local fd = assert(io.open(path, 'rb'))
  local bytes = fd:read('*a')
  fd:close()
  return bytes
end

local function spew(path, bytes)
  local fd = assert(io.open(path, 'wb'))
  fd:write(bytes)
  -- Explicitly, not on collection: nvim opens these back in the same
  -- process and an unflushed copy reads as a truncated spell file.
  fd:close()
end

local function patch(bytes, edits)
  for at, byte in pairs(edits) do
    bytes = bytes:sub(1, at - 1) .. string.char(byte) .. bytes:sub(at + 1)
  end
  return bytes
end

-- Header layout the reader walks: six magic bytes, one version byte, then
-- the eight-byte timestamp that has to equal the one the .spl carries.
local DAMAGE = {
  { 'magic', { [1] = 0x58 } },
  { 'oldversion', { [7] = 0 } },
  { 'newversion', { [7] = 99 } },
  { 'timestamp', { [15] = 0 } },
  { 'truncated', {} },
}

-- The copies live outside the corpus tree: the shell wrapper hashes every
-- .spl and .sug it finds under the corpus, and these are damaged on
-- purpose, so they belong to the read report and not to the byte oracle.
local saldir = corpus .. '/sal/spell'
local dmgdir = corpus .. '/../sugdamage'
vim.fn.mkdir(dmgdir, 'p')
for _, damage in ipairs(DAMAGE) do
  local label, edits = damage[1], damage[2]
  local name = 'dmg' .. label
  emit('=== case sug-' .. label)
  local bytes = patch(slurp(saldir .. '/sal.utf-8.sug'), edits)
  if label == 'truncated' then
    bytes = bytes:sub(1, 20)
  end
  spew(dmgdir .. '/' .. name .. '.utf-8.sug', bytes)
  spew(dmgdir .. '/' .. name .. '.utf-8.spl', slurp(saldir .. '/sal.utf-8.spl'))
  exec('enew!')
  vim.wo.spell = true
  exec('lcd ' .. vim.fn.fnameescape(dmgdir))
  emit('--- spelllang', exec(('setlocal spelllang=%s.utf-8.spl'):format(name)))
  -- The rejection is reported when the suggestion runs, not when the
  -- language loads: the .sug is opened lazily by spell_suggest_intern.
  emit(exec('echo string(spellsuggest("fone", 10))'))
  -- A second call proves the failure is remembered rather than retried.
  emit(exec('echo string(spellsuggest("fone", 10))'))
end

-- --- the shipped dictionary ---------------------------------------------
-- runtime/spell/en.utf-8.spl is the only real-world .spl in the tree and
-- the only one exercising the reader at scale (621 KB, full SAL tables).
-- It is read-only input, so it belongs to the read oracle only.
emit('=== case en-shipped')
exec('enew!')
vim.wo.spell = true
emit('--- spelllang', exec('setlocal spelllang=en'))
emit('--- spellinfo')
emit(exec('spellinfo'))
emit('--- verdicts')
for _, word in ipairs(lines(corpus .. '/../enwords')) do
  local ok, res = pcall(vim.fn.spellbadword, word)
  emit(word, '->', ok and show(res) or ('ERROR ' .. scrub(tostring(res))))
end
emit('--- suggestions')
for _, mode in ipairs(SUGGEST_MODES) do
  vim.o.spellsuggest = mode
  for _, word in ipairs(lines(corpus .. '/../ensugs')) do
    for _, cap in ipairs({ 0, 1 }) do
      local ok, res = pcall(vim.fn.spellsuggest, word, 25, cap)
      emit(
        mode,
        'cap=' .. cap,
        word,
        '->',
        ok and show(res) or ('ERROR ' .. scrub(tostring(res)))
      )
    end
  end
end
