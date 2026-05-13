$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$artifactDir = Join-Path $repoRoot 'artifacts\windows-trial'
New-Item -ItemType Directory -Force -Path $artifactDir | Out-Null

function Get-SseTranslationText {
  param(
    [Parameter(Mandatory = $true)][string]$ResponseText
  )

  $builder = [System.Text.StringBuilder]::new()
  foreach ($rawLine in ($ResponseText -split "`r?`n")) {
    $line = $rawLine.Trim()
    if (-not $line.StartsWith('data: ')) { continue }

    $payload = $line.Substring(6)
    if ($payload -eq '[DONE]') { break }

    $json = $payload | ConvertFrom-Json
    foreach ($choice in $json.choices) {
      if ($null -ne $choice.delta.content -and $choice.delta.content.Length -gt 0) {
        [void]$builder.Append($choice.delta.content)
      }
    }
  }

  return $builder.ToString()
}

function Invoke-ProviderSmoke {
  param(
    [Parameter(Mandatory = $true)][string]$Name,
    [Parameter(Mandatory = $true)][string]$BaseUrl,
    [Parameter(Mandatory = $true)][string]$Model,
    [Parameter(Mandatory = $true)][hashtable]$Headers,
    [Parameter(Mandatory = $true)][string]$Text,
    [int]$TimeoutSeconds = 30
  )

  $body = @{
    model = $Model
    messages = @(
      @{
        role = 'system'
        content = 'You are a professional translator. Translate the following text into Chinese. Output ONLY the translated text.'
      },
      @{
        role = 'user'
        content = $Text
      }
    )
    temperature = 0.3
    stream = $true
  } | ConvertTo-Json -Depth 6

  $uri = ('{0}/chat/completions' -f $BaseUrl.TrimEnd('/'))
  $startedAt = Get-Date
  $response = Invoke-WebRequest -Uri $uri -Method Post -ContentType 'application/json' -Headers $Headers -Body $body -TimeoutSec $TimeoutSeconds
  $translation = Get-SseTranslationText -ResponseText $response.Content

  if ([string]::IsNullOrWhiteSpace($translation)) {
    throw "$Name smoke test returned an empty translation."
  }

  return [pscustomobject]@{
    Provider = $Name
    BaseUrl = $BaseUrl
    Model = $Model
    DurationSeconds = [math]::Round(((Get-Date) - $startedAt).TotalSeconds, 2)
    Translation = $translation
    Passed = $true
  }
}

$results = @()

try {
  if ([string]::IsNullOrWhiteSpace($env:AURA_SMOKE_DEEPSEEK_API_KEY)) {
    throw 'AURA_SMOKE_DEEPSEEK_API_KEY is required for the DeepSeek smoke test.'
  }

  $results += Invoke-ProviderSmoke `
    -Name 'DeepSeek' `
    -BaseUrl 'https://api.deepseek.com' `
    -Model 'deepseek-chat' `
    -Headers @{ Authorization = "Bearer $($env:AURA_SMOKE_DEEPSEEK_API_KEY)" } `
    -Text 'Good morning. This is a release smoke test.'

  $ollamaBaseUrl = if ([string]::IsNullOrWhiteSpace($env:AURA_SMOKE_OLLAMA_BASE_URL)) {
    'http://localhost:11434'
  } else {
    $env:AURA_SMOKE_OLLAMA_BASE_URL
  }
  $ollamaModel = if ([string]::IsNullOrWhiteSpace($env:AURA_SMOKE_OLLAMA_MODEL)) {
    'qwen2.5'
  } else {
    $env:AURA_SMOKE_OLLAMA_MODEL
  }

  $results += Invoke-ProviderSmoke `
    -Name 'Ollama' `
    -BaseUrl $ollamaBaseUrl `
    -Model $ollamaModel `
    -Headers @{} `
    -Text 'Good morning. This is a release smoke test.'

  $output = [pscustomobject]@{
    GeneratedAt = (Get-Date).ToString('s')
    Passed = $true
    Results = $results
  }
  $outputPath = Join-Path $artifactDir 'smoke-providers.json'
  $output | ConvertTo-Json -Depth 6 | Set-Content -Path $outputPath -Encoding UTF8
  $results | Format-Table -AutoSize
  Write-Host "Smoke results written to $outputPath"
} catch {
  $failure = [pscustomobject]@{
    GeneratedAt = (Get-Date).ToString('s')
    Passed = $false
    Error = $_.Exception.Message
    Results = $results
  }
  $outputPath = Join-Path $artifactDir 'smoke-providers.json'
  $failure | ConvertTo-Json -Depth 6 | Set-Content -Path $outputPath -Encoding UTF8
  Write-Error $_.Exception.Message
  exit 1
}
