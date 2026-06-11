" Lua keymap callback invocation regression smoke test.
"
" Guards against E117: Unknown function: <lambda>N when VimL calls a Lua
" callback that was stored as a VAR_FUNC (register_luafunc lambda).
"
" Root cause: typval_exec_lua_callable in exec.rs used FCERR_NONE = 0 (wrong),
" but call_func in funccal.rs used FCERR_NONE = 5 (correct per C userfunc.h
" FnameTransError enum). So call_func saw return value 0 = FCERR_UNKNOWN and
" emitted E117 despite finding the ufunc_T in func_hashtab.
"
" Fixed in src/nvim-rs/lua/src/exec.rs: FCERR_NONE 0->5, FCERR_OTHER 10->6.
"
" IMPORTANT: Must complete within the timeout enforced by just smoke-test-keymap
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
" Setup: register a Lua callback mapping (must be done from Lua).
" ============================================================================

lua GlobalCount = 0
lua vim.api.nvim_set_keymap('n', 'smoke_zzz', '', {callback = function() GlobalCount = GlobalCount + 1 end})

" ============================================================================
" Test 1: maparg(...).callback() from VimL — the primary E117 regression.
" Before the fix this emitted E117: Unknown function: <lambda>N because
" typval_exec_lua_callable returned FCERR_NONE=0 (wrong), which call_func
" treated as FCERR_UNKNOWN=0 and called rs_user_func_error.
" ============================================================================

try
  call maparg('smoke_zzz', 'n', v:false, v:true).callback()
  if luaeval('GlobalCount') ==# 1
    call s:ok('maparg-callback-viml')
  else
    call s:fail('maparg-callback-viml', 'callback ran but GlobalCount=' .. luaeval('GlobalCount') .. ' (expected 1)')
  endif
catch /E117/
  call s:fail('maparg-callback-viml', 'got E117: Unknown function (lambda evicted from func_hashtab before fix)')
catch
  call s:fail('maparg-callback-viml', 'unexpected error: ' .. v:exception)
endtry

" ============================================================================
" Test 2: nvim_get_keymap() dict .callback() from VimL.
" Find the smoke_zzz entry by lhs.
" ============================================================================

let s:found_idx = -1
let s:maps = nvim_get_keymap('n')
for s:i in range(len(s:maps))
  if s:maps[s:i]['lhs'] ==# 'smoke_zzz'
    let s:found_idx = s:i
    break
  endif
endfor

if s:found_idx >=# 0
  try
    call s:maps[s:found_idx].callback()
    if luaeval('GlobalCount') ==# 2
      call s:ok('nvim_get_keymap-callback-viml')
    else
      call s:fail('nvim_get_keymap-callback-viml', 'callback ran but GlobalCount=' .. luaeval('GlobalCount') .. ' (expected 2)')
    endif
  catch /E117/
    call s:fail('nvim_get_keymap-callback-viml', 'got E117: Unknown function (lambda evicted from func_hashtab before fix)')
  catch
    call s:fail('nvim_get_keymap-callback-viml', 'unexpected error: ' .. v:exception)
  endtry
else
  call s:fail('nvim_get_keymap-callback-viml', 'smoke_zzz mapping not found in nvim_get_keymap("n")')
endif

" ============================================================================
" Test 3: VAR_PARTIAL lambda (get_lambda_tv) still works — no regression.
" ============================================================================

let s:F = {-> 42}
try
  let s:result = s:F()
  if s:result ==# 42
    call s:ok('partial-lambda-still-works')
  else
    call s:fail('partial-lambda-still-works', 'expected 42, got ' .. s:result)
  endif
catch
  call s:fail('partial-lambda-still-works', 'unexpected error: ' .. v:exception)
endtry

" ============================================================================
" Report results
" ============================================================================

if len(s:errors) == 0
  echomsg 'keymap-smoke: ALL ' .. s:pass_count .. ' tests passed'
else
  for s:err in s:errors
    echoerr 'FAIL: ' .. s:err
  endfor
  echoerr 'keymap-smoke: ' .. len(s:errors) .. ' FAILED, ' .. s:pass_count .. ' passed'
  cquit!
endif

qall!
