" buf_smoke.vim — regression guard for buffer-local variable access via
" getbufvar()/setbufvar() and b:changedtick.
"
" Regression: TV_SIZE = 24 (should be 16) in vars/src/viml_funcs.rs caused
" E908 "Using an invalid value as a String" on all getbufvar() / setbufvar()
" calls involving the varname offset calculation, and a null-pointer deref in
" rs_list_arg_vars when :let b: was executed.  Both are fixed; this file keeps
" them fixed.
"
" IMPORTANT: This script must complete within the timeout enforced by the
" justfile target.  If it hangs the timeout will kill the process.

let s:errors = []
let s:pass_count = 0

function! s:ok(name)
  let s:pass_count += 1
endfunction

function! s:fail(name, msg)
  call add(s:errors, a:name .. ': ' .. a:msg)
endfunction

" ============================================================================
" Test 1: getbufvar returns a string variable without E908
" ============================================================================

let s:bufnr = bufnr('%')
call setbufvar(s:bufnr, 'myvar', 'hello')
let s:val = getbufvar(s:bufnr, 'myvar')

if s:val ==# 'hello'
  call s:ok('getbufvar-string')
else
  call s:fail('getbufvar-string', 'expected "hello", got: ' .. string(s:val))
endif

" ============================================================================
" Test 2: setbufvar + getbufvar round-trip with a number
" ============================================================================

call setbufvar(s:bufnr, 'numvar', 42)
let s:num = getbufvar(s:bufnr, 'numvar')

if s:num == 42
  call s:ok('getbufvar-number')
else
  call s:fail('getbufvar-number', 'expected 42, got: ' .. string(s:num))
endif

" ============================================================================
" Test 3: getbufvar returns default when variable is absent
" ============================================================================

let s:def = getbufvar(s:bufnr, '__no_such_var__', 'default')

if s:def ==# 'default'
  call s:ok('getbufvar-default')
else
  call s:fail('getbufvar-default', 'expected "default", got: ' .. string(s:def))
endif

" ============================================================================
" Test 4: b:changedtick is a number and increments on buffer change
" ============================================================================

let s:ct1 = b:changedtick

call append(0, 'new line')

let s:ct2 = b:changedtick

if type(s:ct1) == v:t_number && s:ct2 > s:ct1
  call s:ok('b:changedtick-increments')
else
  call s:fail('b:changedtick-increments',
        \ 'before=' .. string(s:ct1) .. ' after=' .. string(s:ct2))
endif

" ============================================================================
" Test 5: :let b: listing does not crash (null-pointer deref before fix)
" ============================================================================

" Redirect :let b: output so it doesn't pollute messages
let s:let_output = ''
redir => s:let_output
silent let b:
redir END

if s:let_output =~# 'changedtick'
  call s:ok('let-b-listing')
else
  call s:fail('let-b-listing', 'expected changedtick in :let b: output, got: ' .. string(s:let_output))
endif

" ============================================================================
" Test 6: getbufvar on a non-existent buffer returns default
" ============================================================================

let s:missing = getbufvar(99999, 'any', 'fallback')

if s:missing ==# 'fallback'
  call s:ok('getbufvar-missing-buffer')
else
  call s:fail('getbufvar-missing-buffer', 'expected "fallback", got: ' .. string(s:missing))
endif

" ============================================================================
" Report results
" ============================================================================

if len(s:errors) == 0
  echomsg 'buf-smoke: ALL ' .. s:pass_count .. ' tests passed'
else
  for err in s:errors
    echoerr 'FAIL: ' .. err
  endfor
  echoerr 'buf-smoke: ' .. len(s:errors) .. ' FAILED, ' .. s:pass_count .. ' passed'
  cquit!
endif

qall!
