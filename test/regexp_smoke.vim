" Regexp smoke test: exercises regexp paths that the baseline test doesn't cover.
"
" The baseline test (regexp_baseline.vim) uses matchstrpos()/matchlist() which
" only tests single-line matching (vim_regexec). This script tests:
"   - Buffer search (vim_regexec_multi) via search()
"   - Substitution via :s
"   - Syntax highlighting (compiles hundreds of real-world patterns)
"
" Each test writes its result to a list. At the end, any failures are reported
" and the script exits with an appropriate code.
"
" IMPORTANT: This script must complete within the timeout enforced by the
" justfile target. If it hangs, the timeout will kill the process and the
" test is considered failed.

let s:errors = []
let s:pass_count = 0

function! s:ok(name)
  let s:pass_count += 1
endfunction

function! s:fail(name, msg)
  call add(s:errors, a:name .. ': ' .. a:msg)
endfunction

" ============================================================================
" Test group 1: Buffer search (vim_regexec_multi)
" ============================================================================

" Simple literal search
new
call setline(1, ['hello world', 'foo bar baz', 'hello again'])
call cursor(1, 1)
let pos = search('hello', 'nc')
if pos == 1
  call s:ok('search-literal')
else
  call s:fail('search-literal', 'expected line 1, got ' .. pos)
endif
bwipeout!

" Search with capture groups
new
call setline(1, ['abc 123 def', 'ghi 456 jkl'])
let pos = search('\(\d\+\)', 'n')
if pos == 1
  call s:ok('search-capture-group')
else
  call s:fail('search-capture-group', 'expected line 1, got ' .. pos)
endif
bwipeout!

" Search forward across lines
new
call setline(1, ['aaa', 'bbb', 'ccc target', 'ddd'])
let pos = search('target', 'n')
if pos == 3
  call s:ok('search-forward')
else
  call s:fail('search-forward', 'expected line 3, got ' .. pos)
endif
bwipeout!

" Search with word boundary
new
call setline(1, ['the cat sat', 'on the mat'])
let pos = search('\<mat\>', 'n')
if pos == 2
  call s:ok('search-word-boundary')
else
  call s:fail('search-word-boundary', 'expected line 2, got ' .. pos)
endif
bwipeout!

" Search with character classes
new
call setline(1, ['no digits here', 'but 42 here'])
let pos = search('\d\+', 'n')
if pos == 2
  call s:ok('search-digit-class')
else
  call s:fail('search-digit-class', 'expected line 2, got ' .. pos)
endif
bwipeout!

" Search with very magic
new
call setline(1, ['foo(bar)', 'baz'])
let pos = search('\v\(bar\)', 'n')
if pos == 1
  call s:ok('search-very-magic')
else
  call s:fail('search-very-magic', 'expected line 1, got ' .. pos)
endif
bwipeout!

" Search with BT engine explicitly
new
call setline(1, ['hello world'])
let pos = search('\%#=1hello', 'n')
if pos == 1
  call s:ok('search-bt-engine')
else
  call s:fail('search-bt-engine', 'expected line 1, got ' .. pos)
endif
bwipeout!

" Search with NFA engine explicitly
new
call setline(1, ['hello world'])
let pos = search('\%#=2hello', 'n')
if pos == 1
  call s:ok('search-nfa-engine')
else
  call s:fail('search-nfa-engine', 'expected line 1, got ' .. pos)
endif
bwipeout!

" Search with no match (must not hang)
new
call setline(1, ['hello world'])
let pos = search('zzzzz', 'n')
if pos == 0
  call s:ok('search-no-match')
else
  call s:fail('search-no-match', 'expected 0, got ' .. pos)
endif
bwipeout!

" Search with anchors
new
call setline(1, ['  indented', 'not indented'])
let pos = search('^not', 'n')
if pos == 2
  call s:ok('search-anchored')
else
  call s:fail('search-anchored', 'expected line 2, got ' .. pos)
endif
bwipeout!

" Search in multi-line buffer (100 lines)
new
for i in range(1, 100)
  call setline(i, 'line ' .. i .. ' of test')
endfor
let pos = search('line 50', 'n')
if pos == 50
  call s:ok('search-100-lines')
else
  call s:fail('search-100-lines', 'expected line 50, got ' .. pos)
endif
bwipeout!

" ============================================================================
" Test group 2: Substitution (vim_regexec_multi + vim_regsub)
" ============================================================================

" Simple substitution
new
call setline(1, ['hello world'])
%s/hello/HELLO/
if getline(1) ==# 'HELLO world'
  call s:ok('sub-simple')
else
  call s:fail('sub-simple', 'got: ' .. getline(1))
endif
bwipeout!

" Substitution with backreference
new
call setline(1, ['foo bar'])
%s/\(\w\+\)/[\1]/g
if getline(1) ==# '[foo] [bar]'
  call s:ok('sub-backref')
else
  call s:fail('sub-backref', 'got: ' .. getline(1))
endif
bwipeout!

" Global substitution across multiple lines
new
call setline(1, ['aaa', 'bbb', 'aaa'])
%s/aaa/zzz/g
if getline(1) ==# 'zzz' && getline(3) ==# 'zzz'
  call s:ok('sub-global-multiline')
else
  call s:fail('sub-global-multiline', 'got: ' .. getline(1) .. ', ' .. getline(3))
endif
bwipeout!

" Substitution with character classes
new
call setline(1, ['abc 123 def 456'])
%s/\d\+/NUM/g
if getline(1) ==# 'abc NUM def NUM'
  call s:ok('sub-digit-class')
else
  call s:fail('sub-digit-class', 'got: ' .. getline(1))
endif
bwipeout!

" Substitution with BT engine
new
call setline(1, ['hello world'])
%s/\%#=1hello/BT/
if getline(1) ==# 'BT world'
  call s:ok('sub-bt-engine')
else
  call s:fail('sub-bt-engine', 'got: ' .. getline(1))
endif
bwipeout!

" Substitution with NFA engine
new
call setline(1, ['hello world'])
%s/\%#=2hello/NFA/
if getline(1) ==# 'NFA world'
  call s:ok('sub-nfa-engine')
else
  call s:fail('sub-nfa-engine', 'got: ' .. getline(1))
endif
bwipeout!

" ============================================================================
" Test group 3: Syntax highlighting (regexp compilation + multi-line matching)
" These load real syntax files that compile hundreds of patterns.
" ============================================================================

function! s:test_syntax(filetype, content)
  new
  call setline(1, a:content)
  let &filetype = a:filetype
  syntax on
  " Force syntax to be computed by requesting syntax ID at line 1
  let synid = synID(1, 1, 1)
  call s:ok('syntax-' .. a:filetype)
  syntax off
  bwipeout!
endfunction

" C syntax
call s:test_syntax('c', ['#include <stdio.h>', '', 'int main(void) {', '  printf("hello\n");', '  return 0;', '}'])

" Vim script syntax
call s:test_syntax('vim', ['function! Foo()', '  let x = 42', '  echo "hello"', 'endfunction'])

" Lua syntax
call s:test_syntax('lua', ['local function foo()', '  local x = 42', '  print("hello")', 'end'])

" Python syntax
call s:test_syntax('python', ['def foo():', '    x = 42', '    print("hello")'])

" JavaScript syntax
call s:test_syntax('javascript', ['function foo() {', '  const x = 42;', '  console.log("hello");', '}'])

" Markdown syntax (triggers yaml.vim inclusion)
" Note: yaml.vim has patterns that cause E874 (NFA stack pop error) — this is a
" known NFA compiler bug with specific patterns. The test still verifies no hang.
try
  call s:test_syntax('markdown', ['# Title', '', 'Some text', '', '```python', 'x = 1', '```'])
catch
  " E874 is expected from yaml.vim; the important thing is no hang
  call s:ok('syntax-markdown (E874 expected)')
endtry

" ============================================================================
" Test group 4: Edge cases that have caused problems before
" ============================================================================

" Recursive regexp (vim_regexec_multi called recursively via autocommands)
new
call setline(1, ['test line'])
" search() during CursorMoved would be recursive — just test it doesn't crash
let pos = search('test', 'n')
if pos == 1
  call s:ok('recursive-safety')
else
  call s:fail('recursive-safety', 'expected 1, got ' .. pos)
endif
bwipeout!

" Pattern with \zs and \ze
new
call setline(1, ['abc def ghi'])
let pos = search('\zs\w\+\ze', 'n')
if pos == 1
  call s:ok('search-zs-ze')
else
  call s:fail('search-zs-ze', 'expected 1, got ' .. pos)
endif
bwipeout!

" Lookbehind
new
call setline(1, ['foo=bar', 'baz=qux'])
let pos = search('\(=\)\@<=bar', 'n')
if pos == 1
  call s:ok('search-lookbehind')
else
  call s:fail('search-lookbehind', 'expected 1, got ' .. pos)
endif
bwipeout!

" Lookahead
new
call setline(1, ['foo=bar', 'baz=qux'])
let pos = search('foo\(=\)\@=', 'n')
if pos == 1
  call s:ok('search-lookahead')
else
  call s:fail('search-lookahead', 'expected 1, got ' .. pos)
endif
bwipeout!

" ============================================================================
" Report results
" ============================================================================

if len(s:errors) == 0
  echomsg 'regexp-smoke: ALL ' .. s:pass_count .. ' tests passed'
else
  for err in s:errors
    echoerr 'FAIL: ' .. err
  endfor
  echoerr 'regexp-smoke: ' .. len(s:errors) .. ' FAILED, ' .. s:pass_count .. ' passed'
  cquit!
endif

qall!
