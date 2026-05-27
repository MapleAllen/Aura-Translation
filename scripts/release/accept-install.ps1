param(
  [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$artifactDir = Join-Path $repoRoot 'artifacts\windows-trial'
New-Item -ItemType Directory -Force -Path $artifactDir | Out-Null

function Get-RegistryInstallEntry {
  $paths = @(
    'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*',
    'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*'
  )

  foreach ($path in $paths) {
    $entry = Get-ItemProperty -Path $path -ErrorAction SilentlyContinue |
      Where-Object { $_.DisplayName -eq 'Aura Translation' } |
      Select-Object -First 1
    if ($null -ne $entry) {
      return $entry
    }
  }

  return $null
}

function Get-OptionalPropertyValue {
  param(
    [Parameter(Mandatory = $true)]$Object,
    [Parameter(Mandatory = $true)][string]$Name
  )

  $property = $Object.PSObject.Properties[$Name]
  if ($null -eq $property) {
    return $null
  }

  return $property.Value
}

function Resolve-ExecutablePathFromCommand {
  param(
    [string]$Command
  )

  if ([string]::IsNullOrWhiteSpace($Command)) {
    return $null
  }

  if ($Command.StartsWith('"')) {
    $parts = $Command -split '"'
    if ($parts.Length -ge 2 -and -not [string]::IsNullOrWhiteSpace($parts[1])) {
      return $parts[1]
    }
    return $null
  }

  $segments = $Command.Split(' ', 2)
  if ($segments.Length -gt 0 -and -not [string]::IsNullOrWhiteSpace($segments[0])) {
    return $segments[0]
  }

  return $null
}

function Resolve-InstallDir {
  param(
    [string[]]$PreferredPaths = @()
  )

  $entry = Get-RegistryInstallEntry
  $candidatePaths = @()

  if ($null -ne $entry) {
    $installLocation = Get-OptionalPropertyValue -Object $entry -Name 'InstallLocation'
    if (-not [string]::IsNullOrWhiteSpace($installLocation)) {
      $candidatePaths += $installLocation
    }

    foreach ($command in @(
      (Get-OptionalPropertyValue -Object $entry -Name 'DisplayIcon'),
      (Get-OptionalPropertyValue -Object $entry -Name 'QuietUninstallString'),
      (Get-OptionalPropertyValue -Object $entry -Name 'UninstallString')
    )) {
      $exePath = Resolve-ExecutablePathFromCommand -Command $command
      if (-not [string]::IsNullOrWhiteSpace($exePath)) {
        $candidatePaths += (Split-Path -Path $exePath -Parent)
      }
    }
  }

  $candidatePaths += $PreferredPaths
  $candidatePaths += @(
    (Join-Path $env:LOCALAPPDATA 'Programs\Aura Translation'),
    (Join-Path $env:LOCALAPPDATA 'Aura Translation'),
    (Join-Path $env:ProgramFiles 'Aura Translation'),
    (Join-Path ${env:ProgramFiles(x86)} 'Aura Translation')
  )

  foreach ($candidate in ($candidatePaths | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Select-Object -Unique)) {
    $exePath = Join-Path $candidate 'aura-translation.exe'
    if ((Test-Path $exePath) -or (Test-Path $candidate)) {
      return $candidate
    }
  }

  return $null
}

function Stop-AuraProcesses {
  Get-Process -Name 'aura-translation' -ErrorAction SilentlyContinue | Stop-Process -Force
}

function Wait-Until {
  param(
    [Parameter(Mandatory = $true)][scriptblock]$Condition,
    [int]$TimeoutSeconds = 20,
    [int]$PollMilliseconds = 500,
    [Parameter(Mandatory = $true)][string]$FailureMessage
  )

  $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
  while ((Get-Date) -lt $deadline) {
    if (& $Condition) {
      return
    }
    Start-Sleep -Milliseconds $PollMilliseconds
  }

  throw $FailureMessage
}

function Invoke-Uninstall {
  param(
    [Parameter(Mandatory = $true)][string]$InstallDir
  )

  $uninstallExe = Join-Path $InstallDir 'uninstall.exe'
  if (Test-Path $uninstallExe) {
    $process = Start-Process -FilePath $uninstallExe -ArgumentList '/S' -Wait -PassThru
    if ($process.ExitCode -ne 0) {
      throw "Uninstaller exited with code $($process.ExitCode)."
    }
    return
  }

  $entry = Get-RegistryInstallEntry
  if ($null -eq $entry) {
    throw 'Could not find an Aura Translation uninstall entry.'
  }

  $quietUninstallString = Get-OptionalPropertyValue -Object $entry -Name 'QuietUninstallString'
  $uninstallString = Get-OptionalPropertyValue -Object $entry -Name 'UninstallString'
  $command = if (-not [string]::IsNullOrWhiteSpace($quietUninstallString)) {
    $quietUninstallString
  } else {
    $uninstallString
  }
  if ([string]::IsNullOrWhiteSpace($command)) {
    throw 'Uninstall command is empty.'
  }

  if ($command.StartsWith('"')) {
    $parts = $command -split '"'
    $exe = $parts[1]
    $args = ($parts[2..($parts.Length - 1)] -join '"').Trim()
  } else {
    $segments = $command.Split(' ', 2)
    $exe = $segments[0]
    $args = if ($segments.Length -gt 1) { $segments[1] } else { '' }
  }

  if ($exe -like '*.msiexec*' -and $args -notmatch '/qn') {
    $args = "$args /qn"
  }
  if ($exe -like '*uninstall.exe*' -and $args -notmatch '/S') {
    $args = "$args /S"
  }

  $process = Start-Process -FilePath $exe -ArgumentList $args -Wait -PassThru
  if ($process.ExitCode -ne 0) {
    throw "Uninstall command exited with code $($process.ExitCode)."
  }
}

$result = [ordered]@{
  GeneratedAt = (Get-Date).ToString('s')
  Passed = $false
  NsisPackage = $null
  MsiPackage = $null
  InstallDir = Join-Path $env:LOCALAPPDATA 'Programs\Aura Translation'
  ConfigPath = Join-Path $env:APPDATA 'aura-translation\config.json'
  Checks = @()
}

Push-Location $repoRoot
try {
  Stop-AuraProcesses

  if (-not $SkipBuild) {
    npm run tauri build | Out-Host
  }

  $nsisPackage = Get-ChildItem -Path 'src-tauri\target\release\bundle\nsis' -Filter '*setup.exe' | Select-Object -First 1
  $msiPackage = Get-ChildItem -Path 'src-tauri\target\release\bundle\msi' -Filter '*.msi' | Select-Object -First 1

  if ($null -eq $nsisPackage) { throw 'NSIS bundle was not generated.' }
  if ($null -eq $msiPackage) { throw 'MSI bundle was not generated.' }

  $result.NsisPackage = $nsisPackage.FullName
  $result.MsiPackage = $msiPackage.FullName
  $result.Checks += [pscustomobject]@{ Name = 'nsis-package-size'; Passed = ($nsisPackage.Length -lt 10MB); Detail = "$([math]::Round($nsisPackage.Length / 1MB, 2)) MB" }
  $result.Checks += [pscustomobject]@{ Name = 'msi-package-size'; Passed = ($msiPackage.Length -lt 10MB); Detail = "$([math]::Round($msiPackage.Length / 1MB, 2)) MB" }

  if ($nsisPackage.Length -ge 10MB -or $msiPackage.Length -ge 10MB) {
    throw 'Installer size gate failed: NSIS and MSI packages must both be below 10 MB.'
  }

  $installProcess = Start-Process -FilePath $nsisPackage.FullName -ArgumentList '/S', '/CURRENTUSER' -Wait -PassThru
  if ($installProcess.ExitCode -ne 0) {
    throw "NSIS installer exited with code $($installProcess.ExitCode)."
  }

  $resolvedInstallDir = $null
  Wait-Until `
    -Condition {
      $script:resolvedInstallDir = Resolve-InstallDir -PreferredPaths @($result.InstallDir)
      $null -ne $script:resolvedInstallDir
    } `
    -FailureMessage 'Install directory was not created after NSIS install.'
  $result.InstallDir = $resolvedInstallDir
  $result.Checks += [pscustomobject]@{ Name = 'install-dir-created'; Passed = $true; Detail = $result.InstallDir }

  $installedExe = Join-Path $result.InstallDir 'aura-translation.exe'
  if (-not (Test-Path $installedExe)) {
    throw 'Installed executable was not found.'
  }

  Start-Process -FilePath $installedExe | Out-Null
  Wait-Until `
    -Condition { @(Get-Process -Name 'aura-translation' -ErrorAction SilentlyContinue).Count -gt 0 } `
    -FailureMessage 'Installed Aura Translation process did not start.'
  $result.Checks += [pscustomobject]@{ Name = 'process-started'; Passed = $true; Detail = $installedExe }

  Wait-Until -Condition { Test-Path $result.ConfigPath } -FailureMessage 'Config file was not created after launch.'
  $result.Checks += [pscustomobject]@{ Name = 'config-created'; Passed = $true; Detail = $result.ConfigPath }

  Stop-AuraProcesses
  Invoke-Uninstall -InstallDir $result.InstallDir

  Wait-Until `
    -Condition { -not (Test-Path $result.InstallDir) -or -not (Test-Path (Join-Path $result.InstallDir 'aura-translation.exe')) } `
    -FailureMessage 'Install directory still contains the application after uninstall.'
  Wait-Until `
    -Condition { @(Get-Process -Name 'aura-translation' -ErrorAction SilentlyContinue).Count -eq 0 } `
    -FailureMessage 'Aura Translation process is still running after uninstall.'

  $result.Checks += [pscustomobject]@{ Name = 'uninstall-clean'; Passed = $true; Detail = 'Executable removed and process stopped.' }
  $result.Passed = $true
} catch {
  $result.Error = $_.Exception.Message
  Write-Error $_.Exception.Message
  exit 1
} finally {
  Stop-AuraProcesses
  $outputPath = Join-Path $artifactDir 'accept-install.json'
  [pscustomobject]$result | ConvertTo-Json -Depth 6 | Set-Content -Path $outputPath -Encoding UTF8
  Pop-Location
  Write-Host "Install acceptance results written to $outputPath"
}
