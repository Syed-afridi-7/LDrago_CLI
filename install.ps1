# LDrago CLI Installer for Windows
# Usage:
#   powershell -ExecutionPolicy Bypass -File install.ps1
#   or from web: irm https://raw.githubusercontent.com/Syed-afridi-7/LDrago_CLI/main/install.ps1 | iex

[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$InstallDir = "$HOME\.ldrago\bin",

    [Parameter()]
    [string]$Version = "latest",

    [Parameter()]
    [switch]$BuildFromSource
)

$ErrorActionPreference = "Stop"

# Resolve and expand target directory
$expandedDir = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($InstallDir)
if (-not (Test-Path -LiteralPath $expandedDir)) {
    Write-Host "Creating installation directory: $expandedDir" -ForegroundColor DarkCyan
    [System.IO.Directory]::CreateDirectory($expandedDir) | Out-Null
}
$installDirResolved = (Resolve-Path -LiteralPath $expandedDir).Path
$targetExe = Join-Path $installDirResolved "ldrago.exe"

$repo = "Syed-afridi-7/LDrago_CLI"
$artifactName = "ldrago-windows-x86_64.zip"

if ($Version -eq "latest") {
    $downloadUrl = "https://github.com/$repo/releases/latest/download/$artifactName"
} else {
    $downloadUrl = "https://github.com/$repo/releases/download/$Version/$artifactName"
}

$installed = $false

# 1. Attempt binary download if not explicitly requested to build from source
if (-not $BuildFromSource) {
    Write-Host "==> Attempting to download pre-built binary from:" -ForegroundColor Cyan
    Write-Host "    $downloadUrl" -ForegroundColor Gray

    $tempZip = Join-Path ([System.IO.Path]::GetTempPath()) "ldrago-win-$([System.Guid]::NewGuid().ToString('N')).zip"
    $tempSha = "$tempZip.sha256"
    $tempExtract = Join-Path ([System.IO.Path]::GetTempPath()) "ldrago-ext-$([System.Guid]::NewGuid().ToString('N'))"

    try {
        try {
            [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12 -bor [System.Net.SecurityProtocolType]::Tls13
        } catch {
            [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12
        }

        Invoke-WebRequest -Uri $downloadUrl -OutFile $tempZip -UseBasicParsing -ErrorAction Stop
        Invoke-WebRequest -Uri "$downloadUrl.sha256" -OutFile $tempSha -UseBasicParsing -ErrorAction SilentlyContinue

        if (Test-Path -LiteralPath $tempZip) {
            # Checksum verification if .sha256 file was downloaded
            if (Test-Path -LiteralPath $tempSha) {
                $expectedHash = ((Get-Content -Path $tempSha -Raw).Trim() -split '\s+')[0].ToLower()
                $actualHash = (Get-FileHash -Path $tempZip -Algorithm SHA256).Hash.ToLower()
                if ($expectedHash -ne $actualHash) {
                    throw "SHA256 checksum verification failed! Expected: $expectedHash, Actual: $actualHash"
                }
                Write-Host "==> Verified SHA256 checksum: $actualHash" -ForegroundColor Green
            }

            Expand-Archive -Path $tempZip -DestinationPath $tempExtract -Force
            $foundExe = Get-ChildItem -Path $tempExtract -Filter "ldrago.exe" -Recurse -File | Select-Object -First 1
            if ($foundExe) {
                Copy-Item -Path $foundExe.FullName -Destination $targetExe -Force
                $installed = $true
                Write-Host "==> Downloaded and installed binary to: $targetExe" -ForegroundColor Green
            }
        }
    } catch {
        Write-Host "--> Pre-built release not found or download failed: $($_.Exception.Message)" -ForegroundColor Yellow
    } finally {
        if (Test-Path -LiteralPath $tempZip) {
            Remove-Item -Path $tempZip -Force -ErrorAction SilentlyContinue
        }
        if (Test-Path -LiteralPath $tempSha) {
            Remove-Item -Path $tempSha -Force -ErrorAction SilentlyContinue
        }
        if (Test-Path -LiteralPath $tempExtract) {
            Remove-Item -Path $tempExtract -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

# 2. Fallback: Build from source if download failed or -BuildFromSource specified
if (-not $installed) {
    Write-Host "==> Falling back to building from source..." -ForegroundColor Cyan

    $localManifest = "crates\ld-cli\Cargo.toml"
    if (Test-Path -LiteralPath $localManifest) {
        Write-Host "==> Local workspace detected ($localManifest). Building with Cargo..." -ForegroundColor Cyan
        if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
            throw "Cargo not found in PATH. Please install Rust and Cargo from https://rustup.rs"
        }

        & cargo build --release --bin ldrago
        $builtExe = if (Test-Path -LiteralPath "target\release\ldrago.exe") {
            "target\release\ldrago.exe"
        } elseif (Test-Path -LiteralPath "target\debug\ldrago.exe") {
            "target\debug\ldrago.exe"
        } else {
            throw "Built binary not found at target\release\ldrago.exe or target\debug\ldrago.exe."
        }

        Copy-Item -Path $builtExe -Destination $targetExe -Force
        $installed = $true
        Write-Host "==> Successfully built and copied binary to: $targetExe" -ForegroundColor Green
    } elseif (Get-Command cargo -ErrorAction SilentlyContinue) {
        Write-Host "==> Building from remote repository via cargo install..." -ForegroundColor Cyan
        $rootDir = Split-Path -Path $installDirResolved -Parent
        & cargo install --git "https://github.com/$repo.git" --bin ldrago --root $rootDir --force
        if ($LASTEXITCODE -ne 0) {
            throw "cargo install failed with exit code $LASTEXITCODE."
        }
        if (Test-Path -LiteralPath $targetExe) {
            $installed = $true
            Write-Host "==> cargo install completed: $targetExe" -ForegroundColor Green
        }
    } else {
        throw "Failed to download pre-built binary and no local workspace or Cargo was found. Please install Rust (https://rustup.rs) or check connectivity."
    }
}

# 3. Validate installation
Write-Host "==> Validating installation..." -ForegroundColor Cyan
if (-not (Test-Path -LiteralPath $targetExe)) {
    throw "Installation failed: $targetExe does not exist."
}

$versionInfo = & $targetExe --version
if ($LASTEXITCODE -ne 0) {
    throw "Validation failed: '$targetExe --version' exited with code $LASTEXITCODE."
}
Write-Host "==> Validation successful: $versionInfo" -ForegroundColor Green

# 4. Check and update User PATH
$userPath = [Environment]::GetEnvironmentVariable("PATH", [System.EnvironmentVariableTarget]::User)
$userPathEntries = if ($userPath) {
    $userPath -split [System.IO.Path]::PathSeparator | ForEach-Object { $_.TrimEnd('\', '/') }
} else {
    @()
}

$normalizedTargetDir = $installDirResolved.TrimEnd('\', '/')
$alreadyInUserPath = $userPathEntries | Where-Object { $_ -ieq $normalizedTargetDir }
$pathAdded = $false

if (-not $alreadyInUserPath) {
    Write-Host "==> Adding $installDirResolved to User PATH..." -ForegroundColor Cyan
    $newUserPath = if ([string]::IsNullOrWhiteSpace($userPath)) {
        $installDirResolved
    } else {
        "$userPath;$installDirResolved"
    }
    try {
        [Environment]::SetEnvironmentVariable("PATH", $newUserPath, [System.EnvironmentVariableTarget]::User)
        $pathAdded = $true
    } catch {
        Write-Warning "Could not persist to User PATH registry: $($_.Exception.Message)"
    }
}

# Update current session PATH so ldrago can be invoked immediately in current terminal
$currentSessionEntries = $env:PATH -split [System.IO.Path]::PathSeparator | ForEach-Object { $_.TrimEnd('\', '/') }
if (-not ($currentSessionEntries | Where-Object { $_ -ieq $normalizedTargetDir })) {
    $env:PATH = "$env:PATH;$installDirResolved"
}

# 5. Formatted success message & getting started tips
Write-Host ""
Write-Host "==========================================================" -ForegroundColor Green
Write-Host "         LDrago CLI Installed Successfully!               " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
Write-Host " Binary location: $targetExe" -ForegroundColor White
Write-Host " Version:         $versionInfo" -ForegroundColor White
if ($pathAdded) {
    Write-Host " PATH status:     Added to User PATH (restart terminal to take effect globally)" -ForegroundColor Yellow
} else {
    Write-Host " PATH status:     Already present in PATH" -ForegroundColor Green
}
Write-Host ""
Write-Host " Getting Started:" -ForegroundColor Cyan
Write-Host "   ldrago doctor      Validate system environment, models, and sandboxes" -ForegroundColor White
Write-Host "   ldrago tui         Launch interactive Terminal User Interface" -ForegroundColor White
Write-Host "   ldrago --help      Explore all commands, flags, and features" -ForegroundColor White
Write-Host "==========================================================" -ForegroundColor Green
Write-Host ""
