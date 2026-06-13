param(
    [string]$Version
)

$ErrorActionPreference = "Stop"

# Determine version
if (-not $Version) {
    if (Test-Path "Cargo.toml") {
        $CargoContent = Get-Content -Raw "Cargo.toml"
        if ($CargoContent -match '\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"') {
            $Version = $Matches[1]
            Write-Host "Read version $Version from Cargo.toml"
        }
    }
}

if (-not $Version) {
    $Version = "0.1.0"
    Write-Warning "Could not determine version, defaulting to $Version"
}

# Normalize version formatting (tag starts with 'v', raw version does not)
$RawVersion = $Version
if ($RawVersion.StartsWith("v")) {
    $RawVersion = $RawVersion.Substring(1)
}
$TagVersion = "v$RawVersion"

Write-Host "Packaging opencode-memory version $TagVersion ($RawVersion)..."

# 1. Compile fresh release binaries
Write-Host "Building release binaries..."
cargo build --release
if ($LASTEXITCODE -ne 0) {
    throw "Cargo build failed"
}

# 2. Setup packaging paths
$PackagingDir = "target/packaging"
$SubDir = Join-Path $PackagingDir "opencode-memory"

if (Test-Path $PackagingDir) {
    Remove-Item -Path $PackagingDir -Recurse -Force -ErrorAction SilentlyContinue
}

New-Item -ItemType Directory -Force -Path $SubDir | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $SubDir "skills") | Out-Null

# 3. Copy files
$SourceExe = "target/release/memory-mcp-server.exe"
$DestExe = Join-Path $SubDir "opencode-memory.exe"
Write-Host "Copying binary..."
Copy-Item -Path $SourceExe -Destination $DestExe -Force

$SourceSkill = "skills/memory-extraction.md"
$DestSkill = Join-Path $SubDir "skills/memory-extraction.md"
if (Test-Path $SourceSkill) {
    Write-Host "Copying skill file..."
    Copy-Item -Path $SourceSkill -Destination $DestSkill -Force
}

# 4. Create ZIP archive
$ZipName = "opencode-memory-$TagVersion-x86_64-pc-windows-msvc.zip"
$ZipPath = Join-Path "target" $ZipName

if (Test-Path $ZipPath) {
    Remove-Item -Path $ZipPath -Force -ErrorAction SilentlyContinue
}

Write-Host "Creating archive $ZipPath..."
Compress-Archive -Path "$SubDir\*" -DestinationPath $ZipPath -Force

# 5. Compute SHA256 checksum
Write-Host "Computing SHA256 checksum..."
$HashInfo = Get-FileHash -Path $ZipPath -Algorithm SHA256
$HashString = $HashInfo.Hash.ToLower()

$ShaFile = "$ZipPath.sha256"
Set-Content -Path $ShaFile -Value "$HashString  $ZipName" -Encoding ascii

Write-Host "=========================================================" -ForegroundColor Green
Write-Host "Packaging completed successfully!" -ForegroundColor Green
Write-Host "Zip Archive: $ZipPath" -ForegroundColor Green
Write-Host "SHA256 Hash: $HashString" -ForegroundColor Green
Write-Host "Checksum File: $ShaFile" -ForegroundColor Green
Write-Host "=========================================================" -ForegroundColor Green
