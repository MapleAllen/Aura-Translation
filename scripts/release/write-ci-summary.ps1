param(
  [Parameter(Mandatory = $true)][string]$Repo,
  [Parameter(Mandatory = $true)][string]$Branch,
  [Parameter(Mandatory = $true)][string]$Sha,
  [Parameter(Mandatory = $true)][string]$RunId,
  [Parameter(Mandatory = $true)][string]$RunUrl,
  [Parameter(Mandatory = $true)][string]$EventName,
  [string]$Phase = 'completed',
  [Parameter(Mandatory = $true)][string]$InstallOutcome,
  [Parameter(Mandatory = $true)][string]$CheckOutcome,
  [Parameter(Mandatory = $true)][string]$RustTestsOutcome,
  [Parameter(Mandatory = $true)][string]$UiTestsOutcome,
  [Parameter(Mandatory = $true)][string]$BuildOutcome,
  [Parameter(Mandatory = $true)][string]$LiveSmokeOutcome,
  [Parameter(Mandatory = $true)][string]$AcceptInstallOutcome,
  [Parameter(Mandatory = $true)][string]$MeasureResourcesOutcome
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$artifactDir = Join-Path $repoRoot 'artifacts\windows-trial'
$summaryPath = Join-Path $artifactDir 'ci-summary.json'
New-Item -ItemType Directory -Force -Path $artifactDir | Out-Null

function Read-JsonFile {
  param(
    [Parameter(Mandatory = $true)][string]$Path
  )

  if (-not (Test-Path $Path)) {
    return $null
  }

  try {
    return Get-Content -Path $Path -Raw | ConvertFrom-Json
  } catch {
    return [pscustomobject]@{
      ParseError = $_.Exception.Message
      Path = $Path
    }
  }
}

$gateOutcomes = [ordered]@{
  install_dependencies = [pscustomobject]@{
    required = $true
    outcome = $InstallOutcome
  }
  check = [pscustomobject]@{
    required = $true
    outcome = $CheckOutcome
  }
  rust_tests = [pscustomobject]@{
    required = $true
    outcome = $RustTestsOutcome
  }
  ui_tests = [pscustomobject]@{
    required = $true
    outcome = $UiTestsOutcome
  }
  build_release_bundles = [pscustomobject]@{
    required = $true
    outcome = $BuildOutcome
  }
  live_provider_smoke = [pscustomobject]@{
    required = ($LiveSmokeOutcome -ne 'skipped')
    outcome = $LiveSmokeOutcome
  }
  install_acceptance = [pscustomobject]@{
    required = $true
    outcome = $AcceptInstallOutcome
  }
  resource_measurement = [pscustomobject]@{
    required = $true
    outcome = $MeasureResourcesOutcome
  }
}

$requiredFailures = @()
if ($Phase -eq 'completed') {
  $requiredFailures = @(
    $gateOutcomes.GetEnumerator() |
      Where-Object { $_.Value.required -and $_.Value.outcome -ne 'success' } |
      ForEach-Object { $_.Key }
  )
}

$summary = [ordered]@{
  generatedAt = (Get-Date).ToUniversalTime().ToString('s') + 'Z'
  repo = $Repo
  branch = $Branch
  sha = $Sha
  runId = $RunId
  runUrl = $RunUrl
  eventName = $EventName
  phase = $Phase
  status = if ($Phase -eq 'started') { 'running' } elseif ($requiredFailures.Count -eq 0) { 'passed' } else { 'failed' }
  passed = if ($Phase -eq 'completed') { ($requiredFailures.Count -eq 0) } else { $null }
  requiredFailures = $requiredFailures
  gateOutcomes = $gateOutcomes
  smokeProviders = Read-JsonFile (Join-Path $artifactDir 'smoke-providers.json')
  acceptInstall = Read-JsonFile (Join-Path $artifactDir 'accept-install.json')
  resourceMetrics = Read-JsonFile (Join-Path $artifactDir 'resource-metrics.json')
}

$summary | ConvertTo-Json -Depth 32 | Set-Content -Path $summaryPath -Encoding UTF8
Write-Host "CI summary written to $summaryPath"
