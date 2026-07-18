-- Modules loaded here will not be cleared and reloaded by the local harness.
-- Keeping these preloaded preserves cross-file setup while still resetting
-- non-helper modules between files.
require('ffi')

-- Nvim.rs: the C fixture helpers (test/unit/fixtures/*.c) were compiled into
-- upstream's test libnvim. Here they live in a shared library built by
-- scripts/run-tests.sh; load it RTLD_GLOBAL so ffi.C resolves its symbols.
do
  local nvim_prg = os.getenv('NVIM_PRG')
  if nvim_prg then
    local so = nvim_prg:gsub('[^/\\]+$', 'unit-fixtures.so')
    -- Keep the handle alive: GC would dlclose() and unmap the symbols.
    local ok, lib = pcall(require('ffi').load, so, true)
    if ok then
      _G._unit_fixtures_lib = lib
    end
  end
end
require('test.unit.testutil')
require('test.unit.preprocess')
