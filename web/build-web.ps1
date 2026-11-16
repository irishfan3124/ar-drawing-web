# Rebuilds the browser versions of AR Drawing from the Rust sources.
# Produces:
#   web/index.html          small page, loads ar_drawing.wasm and mq_js_bundle.js
#   web/AR-Drawing.html     one self-contained file (wasm inlined) for a phone
$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$root = Split-Path -Parent $here

Push-Location $root
try {
    # Cargo prints warnings on stderr; with $ErrorActionPreference = "Stop" that
    # would abort the script mid-build, so judge success by the exit code only.
    $log = & {
        $ErrorActionPreference = 'Continue'
        cargo build --release --target wasm32-unknown-unknown 2>&1
    }
    if ($LASTEXITCODE -ne 0) { $log | Write-Output; throw "wasm build failed" }

    $wasmSrc = Join-Path $root "target\wasm32-unknown-unknown\release\ar_drawing.wasm"
    $bundleSrc = Join-Path $env:USERPROFILE ".cargo\registry\src\index.crates.io-1949cf8c6b5b557f\macroquad-0.4.16\js\mq_js_bundle.js"
    if (-not (Test-Path $bundleSrc)) {
        throw "mq_js_bundle.js not found - run 'cargo fetch' first"
    }

    Copy-Item $wasmSrc (Join-Path $here "ar_drawing.wasm") -Force
    Copy-Item $bundleSrc (Join-Path $here "mq_js_bundle.js") -Force

    $template = [IO.File]::ReadAllText((Join-Path $here "template.html"))
    $bundle = [IO.File]::ReadAllText($bundleSrc)

    # small page: external bundle + external wasm
    $small = $template.Replace("__BUNDLE_TAG__", '<script src="mq_js_bundle.js"></script>').Replace("__WASM_B64__", "")
    [IO.File]::WriteAllText((Join-Path $here "index.html"), $small)

    # single file: bundle and wasm inlined as base64
    $b64 = [Convert]::ToBase64String([IO.File]::ReadAllBytes($wasmSrc))
    $single = $template.Replace("__BUNDLE_TAG__", "<script>`n$bundle`n</script>").Replace("__WASM_B64__", $b64)
    [IO.File]::WriteAllText((Join-Path $here "AR-Drawing.html"), $single)

    Write-Output ("index.html      {0:N0} bytes" -f (Get-Item (Join-Path $here "index.html")).Length)
    Write-Output ("AR-Drawing.html {0:N0} bytes" -f (Get-Item (Join-Path $here "AR-Drawing.html")).Length)
} finally {
    Pop-Location
}
