& {
  $OutDir = '../../packages/wasm/'
  Remove-Item -Recurse -Force $OutDir
  wasm-pack.exe build `
    --scope 'mirascript' `
    --target bundler `
    --out-dir $OutDir `
    $args
}
