param(
  [int]$SampleSeconds = 60,
  [int]$WarmupSeconds = 5
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$artifactDir = Join-Path $repoRoot 'artifacts\windows-trial'
New-Item -ItemType Directory -Force -Path $artifactDir | Out-Null

function Stop-AuraProcesses {
  Get-Process -Name 'aura-translation' -ErrorAction SilentlyContinue | Stop-Process -Force
}

Push-Location $repoRoot
$result = [ordered]@{
  GeneratedAt = (Get-Date).ToString('s')
  Passed = $false
  SampleSeconds = $SampleSeconds
  WarmupSeconds = $WarmupSeconds
  Metrics = $null
  Artifacts = $null
}

try {
  $exe = Get-Item 'src-tauri\target\release\aura-translation.exe'
  $nsis = Get-ChildItem -Path 'src-tauri\target\release\bundle\nsis' -Filter '*setup.exe' | Select-Object -First 1
  $msi = Get-ChildItem -Path 'src-tauri\target\release\bundle\msi' -Filter '*.msi' | Select-Object -First 1

  if ($null -eq $nsis -or $null -eq $msi) {
    throw 'Release bundle artifacts are missing. Run the installer acceptance or tauri build first.'
  }

  Stop-AuraProcesses
  $started = Start-Process -FilePath $exe.FullName -PassThru
  Start-Sleep -Seconds $WarmupSeconds

  $samples = @()
  $startedAt = Get-Date
  $logicalProcessors = [Environment]::ProcessorCount

  for ($i = 0; $i -lt $SampleSeconds; $i++) {
    $proc = Get-Process -Id $started.Id -ErrorAction Stop
    $samples += [pscustomobject]@{
      Timestamp = (Get-Date).ToString('s')
      WorkingSetMB = [math]::Round($proc.WorkingSet64 / 1MB, 2)
      PrivateMemoryMB = [math]::Round($proc.PrivateMemorySize64 / 1MB, 2)
      CpuSeconds = [math]::Round($proc.CPU, 3)
    }
    Start-Sleep -Seconds 1
  }

  $finishedAt = Get-Date
  $firstCpu = $samples[0].CpuSeconds
  $lastCpu = $samples[-1].CpuSeconds
  $cpuPercent = [math]::Round((($lastCpu - $firstCpu) / [math]::Max(1, ($finishedAt - $startedAt).TotalSeconds) / $logicalProcessors) * 100, 2)

  $maxWorkingSet = ($samples | Measure-Object -Property WorkingSetMB -Maximum).Maximum
  $avgWorkingSet = [math]::Round(($samples | Measure-Object -Property WorkingSetMB -Average).Average, 2)
  $maxPrivate = ($samples | Measure-Object -Property PrivateMemoryMB -Maximum).Maximum
  $avgPrivate = [math]::Round(($samples | Measure-Object -Property PrivateMemoryMB -Average).Average, 2)

  $result.Artifacts = [pscustomobject]@{
    ReleaseExeMB = [math]::Round($exe.Length / 1MB, 2)
    NsisMB = [math]::Round($nsis.Length / 1MB, 2)
    MsiMB = [math]::Round($msi.Length / 1MB, 2)
  }
  $result.Metrics = [pscustomobject]@{
    MaxWorkingSetMB = $maxWorkingSet
    AvgWorkingSetMB = $avgWorkingSet
    MaxPrivateMemoryMB = $maxPrivate
    AvgPrivateMemoryMB = $avgPrivate
    ApproxCpuPercent = $cpuPercent
    SampleCount = $samples.Count
    Samples = $samples
  }
  $result.Passed = ($maxWorkingSet -le 20)

  if (-not $result.Passed) {
    throw "Idle RSS gate failed. Max Working Set was $maxWorkingSet MB, expected <= 20 MB."
  }
} catch {
  $result.Error = $_.Exception.Message
  Write-Error $_.Exception.Message
  exit 1
} finally {
  Stop-AuraProcesses
  $outputPath = Join-Path $artifactDir 'resource-metrics.json'
  [pscustomobject]$result | ConvertTo-Json -Depth 8 | Set-Content -Path $outputPath -Encoding UTF8
  Pop-Location
  Write-Host "Resource metrics written to $outputPath"
}
