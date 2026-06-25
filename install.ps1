param(
    [string]$Version = "v0.1.0",
    [switch]$FromSource,
    [switch]$SkipBuild,
    [ValidateSet("opencode", "codex", "claude", "all")]
    [string]$Client = "all",
    [switch]$DryRun,
    [switch]$PrintConfig
)

$ErrorActionPreference = "Stop"

# Ensure security protocols support TLS 1.2 for download
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$ServerName = "opencode-memory"
$BinaryName = "opencode-memory.exe"

# Resolve target directory
$UserProfile = if ($env:USERPROFILE) { $env:USERPROFILE } else { [System.Environment]::GetFolderPath('UserProfile') }
$TargetDir = Join-Path $UserProfile ".config\$ServerName\bin"
$StableExe = Join-Path $TargetDir $BinaryName

# ── Print Config (standalone, no binary needed) ─────────────────────────────
if ($PrintConfig) {
    Write-Host "Printing example configuration for client: $Client"
    Write-Host ""

    if ($Client -eq "all" -or $Client -eq "opencode") {
        Write-Host "--- OpenCode (opencode.jsonc) ---"
        Write-Host "Add to $(Join-Path $UserProfile '.config\opencode\opencode.jsonc'):"
        Write-Host '{'
        Write-Host '  "mcp": {'
        Write-Host '    "opencode-memory": {'
        Write-Host '      "type": "local",'
        Write-Host "      `"command`": [`"$StableExe`"],"
        Write-Host '      "enabled": true,'
        Write-Host '      "timeout": 120000,'
        Write-Host '      "environment": {}'
        Write-Host '    }'
        Write-Host '  }'
        Write-Host '}'
        Write-Host ""
    }

    if ($Client -eq "all" -or $Client -eq "codex") {
        Write-Host "--- Codex (config.toml) ---"
        Write-Host "Add to $(Join-Path $UserProfile '.codex\config.toml'):"
        $CleanPath = $StableExe -replace '\\', '/'
        Write-Host "[mcp_servers.opencode-memory]"
        Write-Host "command = `"$CleanPath`""
        Write-Host "args = []"
        Write-Host ""
    }

    if ($Client -eq "all" -or $Client -eq "claude") {
        Write-Host "--- Claude (.mcp.json) ---"
        Write-Host "Add to $(Join-Path $UserProfile '.claude\.mcp.json'):"
        Write-Host '{'
        Write-Host '  "mcpServers": {'
        Write-Host '    "opencode-memory": {'
        Write-Host '      "command": "'"$StableExe"'",'
        Write-Host '      "args": [],'
        Write-Host '      "disabled": false,'
        Write-Host '      "autoApprove": []'
        Write-Host '    }'
        Write-Host '  }'
        Write-Host '}'
        Write-Host ""
    }

    Write-Host "Copy these snippets into your agent config files."
    return
}

# ── Dry Run (standalone, no binary needed) ──────────────────────────────────
if ($DryRun) {
    Write-Host "DRY RUN — no changes will be made"
    Write-Host "Binary: $StableExe"
    Write-Host "Would configure client(s): $Client"
    Write-Host ""
    Write-Host "Run without -DryRun to apply changes."
    return
}

Write-Host "Installing $ServerName..."

$SourceExe = $null

# If not forcing build from source, attempt download first
if (-not $FromSource) {
    # Normalize version tags (always prefix with 'v' if it looks like a version number and doesn't have it)
    $TagVersion = $Version
    if (-not $TagVersion.StartsWith("v")) {
        $TagVersion = "v$TagVersion"
    }

    $ZipName = "$ServerName-$TagVersion-x86_64-pc-windows-msvc.zip"
    $Url = "https://github.com/stevenke1981/memlong/releases/download/$TagVersion/$ZipName"
    
    $TempDir = Join-Path $env:TEMP "opencode-memory-install-$(Get-Random)"
    New-Item -ItemType Directory -Force -Path $TempDir | Out-Null
    $ZipPath = Join-Path $TempDir $ZipName

    Write-Host "Attempting to download release version $TagVersion from $Url..."
    try {
        if (Get-Command "curl.exe" -ErrorAction SilentlyContinue) {
            curl.exe -L -o $ZipPath $Url
        } else {
            Invoke-WebRequest -Uri $Url -OutFile $ZipPath -UseBasicParsing
        }
        
        if (Test-Path $ZipPath) {
            Write-Host "Extracting $ZipName..."
            $ExtractDir = Join-Path $TempDir "extracted"
            Expand-Archive -Path $ZipPath -DestinationPath $ExtractDir -Force
            
            # Find the binary in the extracted files
            $ExtractedExe = Get-ChildItem -Path $ExtractDir -Filter "*.exe" -Recurse | Select-Object -First 1
            if ($ExtractedExe) {
                $SourceExe = $ExtractedExe.FullName
                Write-Host "Successfully downloaded and extracted binary: $SourceExe"
            } else {
                Write-Warning "Could not find any executable in the downloaded archive."
            }
        }
    } catch {
        Write-Warning "Failed to download release asset: $($_.Exception.Message)"
    }

    if (-not $SourceExe) {
        Write-Warning "Falling back to source compilation path."
        $FromSource = $true
    }
}

# Source build path
if ($FromSource) {
    if (-not $SkipBuild) {
        Write-Host "Building $ServerName from source..."
        $CargoCmd = "cargo build --release"
        Write-Host "Running: $CargoCmd"
        Invoke-Expression $CargoCmd
        if ($LASTEXITCODE -ne 0) {
            throw "Cargo build failed"
        }
    } else {
        Write-Host "Skipping cargo build as requested (-SkipBuild)."
    }

    # Verify built binary
    $BuiltExe = "target/release/memory-mcp-server.exe"
    if (-not (Test-Path $BuiltExe)) {
        throw "Built binary not found at $BuiltExe. Please ensure you built it first or run without -SkipBuild."
    }
    $SourceExe = (Resolve-Path $BuiltExe).Path
    Write-Host "Using compiled binary at $SourceExe"
}

# Ensure target folder exists
if (-not (Test-Path $TargetDir)) {
    New-Item -ItemType Directory -Force -Path $TargetDir | Out-Null
}

# Step 3: Handle locked binary side-by-side versioning fallback
$InstalledExe = $StableExe
$IsStableCopied = $false

Write-Host "Installing executable to target path..."
try {
    Copy-Item -Path $SourceExe -Destination $StableExe -Force -ErrorAction Stop
    $IsStableCopied = $true
    Write-Host "Copied binary to stable path: $StableExe"
} catch {
    # Copy failed (e.g. locked or access denied)
    $ErrMessage = $_.Exception.Message
    Write-Warning "Could not copy to stable path $StableExe ($ErrMessage)."
    
    $TagVersion = $Version
    if (-not $TagVersion.StartsWith("v")) {
        $TagVersion = "v$TagVersion"
    }
    
    $VersionedName = "$ServerName-$TagVersion.exe"
    $VersionedExe = Join-Path $TargetDir $VersionedName
    
    Write-Warning "Installing versioned binary side-by-side: $VersionedExe"
    try {
        Copy-Item -Path $SourceExe -Destination $VersionedExe -Force -ErrorAction Stop
        $InstalledExe = $VersionedExe
        Write-Host "Copied binary to versioned path: $InstalledExe"
    } catch {
        $Timestamp = (Get-Date).ToString("yyyyMMdd-HHmmss")
        $TimestampedName = "$ServerName-$TagVersion-$Timestamp.exe"
        $TimestampedExe = Join-Path $TargetDir $TimestampedName
        Write-Warning "Installing timestamped binary side-by-side: $TimestampedExe"
        Copy-Item -Path $SourceExe -Destination $TimestampedExe -Force
        $InstalledExe = $TimestampedExe
        Write-Host "Copied binary to timestamped path: $InstalledExe"
    }
}

# Step 4: Configure agents by running the installed binary
$InstallArgs = @("install")
if ($Client -ne "all") {
    $InstallArgs += "--client"
    $InstallArgs += $Client
}

Write-Host "Running configuration installer from $InstalledExe..."
& $InstalledExe $InstallArgs
if ($LASTEXITCODE -ne 0) {
    throw "Installed binary failed to configure system settings (exit code $LASTEXITCODE)."
}

Write-Host "=========================================================" -ForegroundColor Green
Write-Host "$ServerName has been installed successfully!" -ForegroundColor Green
Write-Host "Installed Executable Path: $InstalledExe" -ForegroundColor Green
if (-not $IsStableCopied) {
    Write-Host "Note: Installed as a side-by-side versioned binary because the stable executable was in use." -ForegroundColor Yellow
}
Write-Host "Please restart your OpenCode or Codex agent to reload changes." -ForegroundColor Green
Write-Host "=========================================================" -ForegroundColor Green
