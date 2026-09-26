# Every check CLAUDE.md asks for before a push, in one run, stopping at the
# first failure: formatting, clippy, the tests (the GPU ones included, which
# `cargo test` skips), and the OpenSpec validation.
#
#   powershell -ExecutionPolicy Bypass -File tools\check_all.ps1
#
# Uses the `fast` profile, which shares its build with `run.bat`. It needs the
# MSVC C++ build tools (the linker and the Windows SDK libraries).

$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)

function Step($name, [scriptblock]$body) {
    Write-Host "== $name" -ForegroundColor Cyan
    & $body
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAILED: $name" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

Step 'format' { cargo fmt --all -- --check }
Step 'clippy' { cargo clippy --profile fast --locked --workspace --all-targets -- -D warnings }
Step 'tests' { cargo test --profile fast --locked --workspace }
Step 'GPU tests' { cargo test --profile fast --locked -p pbd-app --lib -- --ignored }
Step 'openspec' { npx -y @fission-ai/openspec validate --all }
Write-Host 'All checks passed.' -ForegroundColor Green
