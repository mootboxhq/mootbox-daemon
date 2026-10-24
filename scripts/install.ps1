[CmdletBinding()]
param(
    [Uri]$MootboxUrl = 'https://mootbox.zbib.dev',
    [Guid]$Workspace = [Guid]::Empty,
    [Guid]$Project = [Guid]::Empty
)

$ErrorActionPreference = 'Stop'
if (-not [Environment]::Is64BitOperatingSystem) {
    throw 'Mootbox requires 64-bit Windows 10 or later.'
}
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    throw 'Install Git for Windows, open a new PowerShell terminal in your repository, and run this command again.'
}
git rev-parse --show-toplevel 2>$null | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw 'Open PowerShell in your Git repository folder, then run this command again.'
}

$installDirectory = Join-Path $env:USERPROFILE '.mootbox\bin'
$executable = Join-Path $installDirectory 'mootbox.exe'
if (Test-Path -LiteralPath $executable) {
    & $executable update
    if ($LASTEXITCODE -ne 0) { throw 'Mootbox could not update. Check the error above and try again.' }
} else {
    $releaseUrl = 'https://github.com/mootboxhq/mootbox-daemon/releases/latest/download'
    $asset = 'mootbox-windows-x64.zip'
    $temporary = Join-Path ([IO.Path]::GetTempPath()) ([Guid]::NewGuid().ToString())
    New-Item -ItemType Directory -Path $temporary | Out-Null
    try {
        $archive = Join-Path $temporary $asset
        Write-Host 'Downloading Mootbox for Windows...'
        Invoke-WebRequest -UseBasicParsing -Uri "$releaseUrl/$asset" -OutFile $archive
        $checksumFile = Join-Path $temporary 'SHA256SUMS'
        Invoke-WebRequest -UseBasicParsing -Uri "$releaseUrl/SHA256SUMS" -OutFile $checksumFile
        $sums = [IO.File]::ReadAllText($checksumFile)
        $expected = $null
        foreach ($line in ($sums -split "`n")) {
            $parts = $line.Trim() -split '\s+'
            if ($parts.Count -eq 2 -and $parts[1] -ceq $asset) { $expected = $parts[0]; break }
        }
        if (-not $expected) { throw "SHA256SUMS has no checksum for $asset." }
        $actual = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash
        if ($actual -ine $expected) { throw 'The Mootbox download failed checksum verification. Run this command again.' }
        $unpacked = Join-Path $temporary 'unpacked'
        Expand-Archive -LiteralPath $archive -DestinationPath $unpacked
        $download = Join-Path $unpacked 'mootbox.exe'
        if (-not (Test-Path -LiteralPath $download -PathType Leaf)) { throw 'The download contains no mootbox.exe.' }
        & $download --version
        if ($LASTEXITCODE -ne 0) { throw 'This Mootbox download could not run on your Windows installation.' }
        New-Item -ItemType Directory -Force -Path $installDirectory | Out-Null
        Copy-Item -LiteralPath $download -Destination $executable
    } finally {
        Remove-Item -LiteralPath $temporary -Recurse -Force
    }
}

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (($userPath -split ';') -notcontains $installDirectory) {
    [Environment]::SetEnvironmentVariable('Path', "$installDirectory;$userPath", 'User')
}
if (($env:Path -split ';') -notcontains $installDirectory) { $env:Path = "$installDirectory;$env:Path" }
$arguments = @('add', '--mootbox-url', $MootboxUrl.AbsoluteUri.TrimEnd('/'))
if ($Workspace -ne [Guid]::Empty) { $arguments += @('--workspace', $Workspace.ToString()) }
if ($Project -ne [Guid]::Empty) { $arguments += @('--project', $Project.ToString()) }
Write-Host 'Connecting this repository. Confirm the device in your browser, then return here to finish.'
& $executable @arguments
if ($LASTEXITCODE -ne 0) { throw 'Mootbox setup did not finish. Check the message above, then run the same command again.' }
