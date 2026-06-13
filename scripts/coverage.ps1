# coverage.ps1 — Run coverage measurement for memory-core
#
# Prerequisites:
#   cargo install cargo-llvm-cov
#
# Usage:
#   .\scripts\coverage.ps1          # full coverage report (HTML + summary)
#   .\scripts\coverage.ps1 -Quick   # fast terminal-only report
#   .\scripts\coverage.ps1 -Open    # open HTML report in browser
#
# Target modules (from spec):
#   extraction, consolidation, retrieval, storage, mcp-server

param(
    [switch]$Quick,
    [switch]$Open
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $PSCommandPath)

# Check required tools
$has_llvm_cov = $null -ne (Get-Command cargo-llvm-cov -ErrorAction SilentlyContinue)

if (-not $has_llvm_cov) {
    Write-Host "⚠ cargo-llvm-cov is not installed." -ForegroundColor Yellow
    Write-Host "  Install it with: cargo install cargo-llvm-cov"
    Write-Host ""
    Write-Host "  Then run this script again."
    exit 1
}

Push-Location $root
try {
    if ($Quick) {
        # Fast terminal-only report (skips HTML generation)
        cargo llvm-cov --no-report --workspace --exclude memory-cli
        cargo llvm-cov report --workspace --exclude memory-cli
    } else {
        # Full report with HTML
        $report_dir = "$root\target\cov"
        cargo llvm-cov --no-report --workspace --exclude memory-cli
        cargo llvm-cov report --html --output-dir $report_dir --workspace --exclude memory-cli

        Write-Host "Coverage report generated at: $report_dir\index.html" -ForegroundColor Green

        if ($Open) {
            Start-Process "$report_dir\index.html"
        }
    }
} finally {
    Pop-Location
}
