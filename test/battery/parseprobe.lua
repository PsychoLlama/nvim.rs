-- Differential probe for the Ex command-line *parser* (ex_docmd.rs).
--
-- `nvim_parse_cmd()` runs the whole parsing core with none of the effects:
-- `parse_cmdline` -> `parse_command_modifiers` -> `parse_cmd_address` ->
-- `find_ex_command` -> the bang/register/count/argument scan, and it hands
-- back everything each of those decided.  So one case is one command line and
-- the recording is the entire parse tree plus the exact error text.
--
-- Being pure, this probe is cheap enough to run at every build, and it is the
-- only oracle that separates "parsed wrong" from "executed wrong".
--
--   ex-run.sh <nvim> <runtime> <outdir> <label> parse
--   PROBE_LIST=1 ... > cases.txt
--
-- Rehomed from the phase-14 scratchpad at B17-5.  Its rows are byte-identical
-- to the six-month-old REF taken at 341d60cfea: this half of the probe has
-- never moved, and nothing here leaks an environment.  The path scrub below
-- is a no-op today and exists so that a case which *does* name a file cannot
-- silently bake the caller's directory into the baseline.

local out = io.stdout
out:setvbuf('line')

local FROM = tonumber(os.getenv('PROBE_FROM') or '1')
local LIST = os.getenv('PROBE_LIST')

local api = vim.api

local WORK = os.getenv('EX_WORK') or ''
local RT = os.getenv('EX_RUNTIME') or (os.getenv('VIMRUNTIME') or '')
local HOME = os.getenv('HOME') or ''

local function lit(s) return (s:gsub('[%^%$%(%)%%%.%[%]%*%+%-%?]', '%%%1')) end

local function scrub(s)
  s = tostring(s)
  s = s:gsub('0x%x%x%x%x%x%x%x%x+', '0xADDR')
  s = s:gsub('Error in pre%-vimrc command line:\n', '')
  if WORK ~= '' then s = s:gsub(lit(WORK), '<WORK>') end
  if RT ~= '' then s = s:gsub(lit(RT), '<RT>') end
  if HOME ~= '' and HOME ~= '/' then s = s:gsub(lit(HOME), '<HOME>') end
  s = s:gsub('\n', '\\n'):gsub('\r', '\\r'):gsub('\t', '\\t')
  return s
end

--------------------------------------------------------------------------
-- Case list
--------------------------------------------------------------------------

local cases = {}
local seen = {}
local function C(line)
  if seen[line] then return end
  seen[line] = true
  cases[#cases + 1] = line
end

-- 1. Range forms.  Every address kind, every combination of the two halves,
--    every offset spelling, and the degenerate ones.
local ADDRS = {
  '', '.', '$', '%', '0', '1', '5', '7', '99', '2147483647', '2147483648',
  '9223372036854775807', '-9223372036854775808', '99999999999999999999',
  "'a", "'z", "'<", "'>", "'\"", "'[", "']", "'.", "'^", "'q",
  '/brown/', '/nomatch/', '?brown?', '?nomatch?', '\\/', '\\?', '\\&',
  '+', '-', '+3', '-3', '+0', '-0', '.+2', '.-2', '$-1', '$+1',
  "'a+1", "'a-1", '/brown/+1', '/brown/-1', '5+', '5-',
  '\\/+2', '.+++', '.---', '3;', '.,', ',', ';',
}
for _, a in ipairs(ADDRS) do
  C(a .. 'p')
  C(a .. 'print')
  C(a)
end
-- two-address forms, comma and semicolon
local A2 = { '', '.', '$', '1', '5', "'a", '/brown/', '+2', '-2', '0', '99' }
for _, a in ipairs(A2) do
  for _, b in ipairs(A2) do
    C(a .. ',' .. b .. 'p')
    C(a .. ';' .. b .. 'p')
  end
end
-- reversed ranges (the swap prompt), and three-address chains
C('5,1p')
C('$,1p')
C("'z,'ap")
C('5,1,9p')
C('1,2,3,4p')
C('5;1p')

-- 2. Address kinds that are not lines: windows, buffers, arguments, tabs.
for _, a in ipairs({ '', '.', '$', '%', '0', '1', '2', '99', '.+1', '$-1', '1,2', '.,$' }) do
  for _, c in ipairs({ 'wincmd w', 'bdelete', 'argdelete', 'tabnext', 'tabclose',
                       'buffer', 'sbuffer', 'bnext', 'close', 'only',
                       'tabmove', 'tabonly', 'argument', 'next' }) do
    C(a .. c)
  end
end

-- 3. Command modifiers, alone, stacked, abbreviated, and with counts.
local MODS = {
  'silent', 'silent!', 'sil', 'unsilent', 'uns', 'verbose', '5verbose', '0verbose',
  'verbose!', 'browse', 'confirm', 'conf', 'hide', 'keepalt', 'keepa',
  'keepjumps', 'keepj', 'keepmarks', 'kee', 'keeppatterns', 'keepp',
  'lockmarks', 'loc', 'noautocmd', 'noa', 'noswapfile', 'nos', 'sandbox', 'sandb',
  'vertical', 'vert', 'horizontal', 'hor', 'aboveleft', 'abo', 'belowright', 'bel',
  'topleft', 'to', 'botright', 'bo', 'leftabove', 'lefta', 'rightbelow', 'rightb',
  'tab', '3tab', '0tab', 'tabmove', 'filter /x/', 'filter! /x/', 'filter x',
  'legacy',
}
for _, m in ipairs(MODS) do
  C(m .. ' print')
  C(m .. ' echo 1')
  C(m .. ' 1,2print')
  C(m)
end
-- stacks two and three deep
local STACK = { 'silent', 'verbose', '3verbose', 'keeppatterns', 'noautocmd',
                'vertical', 'tab', 'lockmarks', 'sandbox', 'confirm', 'browse',
                'keepjumps', 'topleft', 'filter /x/' }
for _, a in ipairs(STACK) do
  for _, b in ipairs(STACK) do
    C(a .. ' ' .. b .. ' print')
  end
end
for _, a in ipairs({ 'silent', 'vertical', 'tab', 'noautocmd' }) do
  C(a .. ' silent! keeppatterns lockmarks 1,3print')
  C(a .. ' ' .. a .. ' ' .. a .. ' print')
end
-- a modifier where a range belongs and vice versa
C('1,2 silent print')
C('silent 1,2 print')
C('silent1,2print')
C('3silent print')

-- 4. Bang, register, count, and the argument scan.
local CMDS_BANG = { 'write', 'quit', 'wq', 'edit', 'normal', 'global', 'vglobal',
                    'substitute', 'sort', 'redir', 'bdelete', 'set', 'runtime',
                    'nohlsearch', 'undo', 'redo', 'file', 'earlier', 'later' }
for _, c in ipairs(CMDS_BANG) do
  C(c)
  C(c .. '!')
  C(c .. ' arg')
  C(c .. '! arg')
  C(c .. '  spaced   arg  ')
end
-- register + count, the two orders and the illegal spellings
for _, r in ipairs({ 'a', 'z', 'A', '"', '0', '9', '-', '_', '+', '*', '/', ':', '.', '%', '#', '=', '1' }) do
  C('delete ' .. r)
  C('yank ' .. r)
  C('put ' .. r)
  C('delete ' .. r .. ' 3')
  C('yank ' .. r .. ' 3')
  C('1,2delete ' .. r .. ' 5')
end
for _, n in ipairs({ '', '0', '1', '3', '99', '2147483647', '2147483648',
                     '9223372036854775807', '99999999999999999999' }) do
  C('delete ' .. n)
  C('yank ' .. n)
  C('>' .. ' ' .. n)
  C('normal! ' .. n .. 'x')
  C('3,5delete a ' .. n)
  C('buffer ' .. n)
  C('sleep ' .. n .. 'm')
end

-- 5. Command-name resolution: prefixes, ambiguity, unknown, one-letter forms,
--    the `:k`/`:s`/`:d` special cases and the uppercase commands.
local NAMES = {
  'p', 'pr', 'pri', 'prin', 'print', 'printx',
  's', 'su', 'sub', 'subs', 'substitute', 'sm', 'sno', 'sce', 'scr', 'sI',
  'd', 'de', 'del', 'delete', 'dl', 'dp', 'delp', 'dell', 'delel',
  'ddddddddl', 'ddddddddp', 'deletel', 'deletep', 'delxl', 'dxp', 'dxxxxxxxxxxl',
  'k', 'ka', 'kb', 'm', 'mo', 'move', 'ma', 'mark', 't', 'co', 'copy',
  'g', 'g!', 'v', 'gl', 'go', 'norm', 'normal', 'n', 'ne', 'new',
  '&', '&&', '~', '<', '>', '<<', '>>', '=', '==', '!', '!!', '#', '##',
  '*', '@', '@@', '@:', ':', '', ' ', '  ', '"comment', '|', '||',
  'X', 'Next', 'N', 'Print', 'Z', 'ZZ',
  'abcdefgh', 'zzz', 'e', 'ex', 'exi', 'exit', 'x', 'xit', 'xa',
  'wqa', 'wqall', 'xall', 'qa', 'quita', 'quitall',
  'fu', 'func', 'function', 'endfu', 'endfunction',
  'if', 'el', 'else', 'elsei', 'elseif', 'en', 'endif', 'endi',
  'wh', 'while', 'endw', 'endwhile', 'for', 'endfor', 'try', 'cat', 'catch',
  'fina', 'finally', 'endt', 'endtry', 'throw', 'ret', 'return',
  'brea', 'break', 'con', 'continue', 'let', 'unl', 'unlet', 'lockv', 'unlo',
  'ec', 'echo', 'echon', 'echom', 'echomsg', 'echoe', 'echoerr', 'exe', 'execute',
}
for _, n in ipairs(NAMES) do
  C(n)
  C(n .. ' x')
  C('1,2' .. n)
  C(n .. '!')
end

-- 6. Bar separation, comments and the trailing-garbage rules.
for _, l in ipairs({
  'print | print', 'print|print', 'echo 1 | echo 2', 'echo "a|b"',
  'normal! ix|y', 'normal ix|y', 'global/a/print | print',
  'substitute/a/b/ | print', 'substitute/a|b/c/', 'set ff=unix | print',
  'map x y|z', 'autocmd BufRead * echo 1 | echo 2',
  'echo 1 " trailing comment', 'print " comment', '" whole line comment',
  'print xyz', 'print!', 'undo 5 extra', 'wincmd', 'wincmd ww',
  'silent! echo 1 | echo 2', '1,2 | 3,4print', 'print |', 'print ||',
  '@a | print', 'k a | print', 'help |', 'help \\|',
}) do C(l) end

-- 7. Leading colons, whitespace, and the empty command with a range.
for _, l in ipairs({
  ':print', '::print', ':::print', '  :  print', '\t print', ':  ',
  ' ', '   ', ':|print', '  1,2  print  ', '\t\t1print',
}) do C(l) end

-- 8. Commands whose argument scan is special: XFILE, NOTRLCOM, CTRLV, ARGOPT.
for _, l in ipairs({
  'edit %', 'edit #', 'edit %:h', 'edit <cfile>', 'edit <afile>', 'edit <sfile>',
  'edit <cword>', 'edit <cWORD>', 'edit <slnum>', 'edit <stack>', 'edit <script>',
  'edit %%', 'edit \\%', 'edit a\\ b', 'edit "a b"', 'edit ~/x', 'edit $HOME/x',
  'edit ++enc=utf-8 f', 'edit ++ff=unix f', 'edit ++bad=keep f', 'edit ++bin f',
  'edit ++nobin f', 'edit ++edit f', 'edit ++p f', 'edit ++bogus f', 'edit ++',
  'edit +5 f', 'edit +/pat f', 'edit +set\\ ff=unix f', 'edit + f',
  'write >> f', 'write >>f', 'write !cmd', 'write ! cmd', 'read !cmd', 'read f',
  'map <C-A> x', 'map <special> a b', 'imap jk <Esc>',
  'autocmd BufRead *.c echo 1', 'command! -nargs=1 Foo echo 1',
  'runtime! plugin/*.vim', 'source $VIMRUNTIME/filetype.vim',
}) do C(l) end

-- 9. :normal, :execute and the re-entrant forms (parse only -- no effect).
for _, l in ipairs({
  'normal! ggdG', 'normal ggdG', '1,3normal! x', 'normal!', 'normal',
  'execute "print"', 'execute "1,2print"', 'exe "silent print"',
  'execute', 'execute ""', 'global/a/normal! x', 'g/a/s/b/c/',
  'g/a/g/b/print', 'v/a/d', 'g!/a/d', 'global! /a/ d',
  '1,2global/x/print', 'g/x/', 'g//print',
}) do C(l) end

-- 10. Malformed and error-producing lines: the E-number is the answer.
for _, l in ipairs({
  '1,2', "'x", "'", '/', '?', '//', '??', '\\', '\\x', '5,', ',5',
  ';', ';;', '.,,.print', '1,2,print', '+++++++++print',
  '99999999999999999999999999999999print',
  'delete zz', 'yank !!', 'put ~~', 'normal!!', 'substitute',
  'k', 'mark', 'mark toolong', 'k ab', "'a,'bp",
  '.!', '1,2!', '!!!', '.=', '<<<<<<<<', '>>>>>>>>',
  'redir', 'redir @', 'redir @a', 'redir @a>', 'redir @a>>', 'redir END',
  'sleep', 'sleep x', 'sleep 5x', 'sleep 1m',
}) do C(l) end

if LIST then
  for i, c in ipairs(cases) do
    out:write(('%d\tparse\t%s\n'):format(i, scrub(c)))
  end
  os.exit(0)
end

--------------------------------------------------------------------------
-- Scene: the buffer the addresses resolve against.
--------------------------------------------------------------------------

api.nvim_command('set nomore report=99999 shortmess+=F belloff=all')
api.nvim_command('set noswapfile nobackup noundofile hidden')
api.nvim_buf_set_lines(0, 0, -1, false, {
  'The quick brown fox jumps over the lazy dog.',
  'second line of the scene buffer',
  'third line with brown in it too',
  'fourth line',
  'fifth line, the cursor starts here',
  'sixth line',
  'seventh line',
  'eighth and last line',
})
vim.fn.cursor(5, 3)
api.nvim_command("normal! majjmz")
api.nvim_command('normal! vjl\27')
vim.fn.setreg('a', 'register a')

--------------------------------------------------------------------------
-- Recording one case
--------------------------------------------------------------------------

-- vim.inspect sorts keys, so the recording is stable; flatten the newlines.
local function flat(v)
  return (vim.inspect(v, { newline = ' ', indent = '' }):gsub('%s+', ' '))
end

for i = FROM, #cases do
  local line = cases[i]
  local ok, res = pcall(api.nvim_parse_cmd, line, {})
  out:write(('%d\tparse\t%s\t%s\t%s\n'):format(
    i, scrub(line), ok and 'ok' or 'E', scrub(ok and flat(res) or res)))
end

vim.cmd('qa!')
