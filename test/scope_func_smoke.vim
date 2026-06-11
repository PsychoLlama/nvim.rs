" Scope-prefix funcref name uppercase-check regression smoke test.
"
" Guards against the bug where rs_var_wrong_func_name only skipped the 'x:'
" scope prefix for w/b/s/t scopes instead of any scope (name[1] == ':').
" Before the fix, `let g:Cb = {-> 1}` would spuriously raise E704.
"
" C reference: bfdf10d870:src/nvim/eval/vars.c:var_wrong_func_name
"   (name[0] != NUL && name[1] == ':') ? name[2] : name[0]
"
" IMPORTANT: Must complete within the timeout enforced by just smoke-test-scope-func
" (15 seconds).

let s:errors = []
let s:pass_count = 0

function! s:ok(name)
  let s:pass_count += 1
endfunction

function! s:fail(name, msg)
  call add(s:errors, a:name .. ': ' .. a:msg)
endfunction

" ============================================================================
" Test 1: g:Cb = funcref — must NOT raise E704
" Before the fix: has_scope_prefix=false (g not in w/b/s/t), check_char='g'
" (lowercase) → spurious E704.
" ============================================================================

try
  let g:Cb = {-> 1}
  call s:ok('g-scope-uppercase-no-E704')
catch /E704/
  call s:fail('g-scope-uppercase-no-E704', 'spurious E704 raised for g:Cb')
endtry

" ============================================================================
" Test 2: Calling g:Cb() returns the expected value.
" ============================================================================

if exists('g:Cb') && g:Cb() ==# 1
  call s:ok('g-scope-funcref-callable')
else
  call s:fail('g-scope-funcref-callable', 'g:Cb did not return 1 or was not set')
endif

" ============================================================================
" Test 3: Unscoped lowercase name still raises E704.
" ============================================================================

let s:got_e704 = 0
try
  let cb = {-> 1}
catch /E704/
  let s:got_e704 = 1
endtry
if s:got_e704
  call s:ok('unscoped-lowercase-raises-E704')
else
  call s:fail('unscoped-lowercase-raises-E704', 'expected E704 for lowercase unscoped funcref name')
endif

" ============================================================================
" Test 4: Unscoped uppercase name (Cb) must NOT raise E704.
" ============================================================================

try
  let Cb = {-> 2}
  call s:ok('unscoped-uppercase-no-E704')
catch /E704/
  call s:fail('unscoped-uppercase-no-E704', 'spurious E704 raised for unscoped Cb')
endtry

" ============================================================================
" Test 5: b:Cb (allowed prefix) must NOT raise E704.
" ============================================================================

try
  let b:Cb = {-> 3}
  call s:ok('b-scope-uppercase-no-E704')
catch /E704/
  call s:fail('b-scope-uppercase-no-E704', 'spurious E704 raised for b:Cb')
endtry

" ============================================================================
" Report results
" ============================================================================

if len(s:errors) == 0
  echomsg 'scope-func-smoke: ALL ' .. s:pass_count .. ' tests passed'
else
  for s:err in s:errors
    echoerr 'FAIL: ' .. s:err
  endfor
  echoerr 'scope-func-smoke: ' .. len(s:errors) .. ' FAILED, ' .. s:pass_count .. ' passed'
  cquit!
endif

qall!
