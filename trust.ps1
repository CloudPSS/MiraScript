#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"

$env:https_proxy = ''
# 触发 2FA 验证
npm trust list --registry=https://registry.npmjs.org/

Get-ChildItem -Path "packages/*" -Directory | ForEach-Object {
  $target = $_.Name
  $packageJson = Get-Content "$_\package.json" -Raw | ConvertFrom-Json

  if ($packageJson.private -eq $true) {
    Write-Host "Skipping $target because it is private"
    return
  }

  Push-Location $_
  try {
    Write-Host "Trusting $target"
    $output = "$(npm trust list --registry=https://registry.npmjs.org/)"

    if ($output -match '\bid:\s+(\S+)') {
      $id = $Matches[1]
    } else {
      $id = $null
    }

    if (-not $id) {
      Write-Host "No trust ID found for $target"
    } else {
      Write-Host "Found trust ID $id for $target, revoking trust"
      npm trust revoke --registry=https://registry.npmjs.org/ --id $id
    }

    npm trust github --allow-publish --allow-stage-publish --file release.yml --env npm --registry https://registry.npmjs.org/ --yes
  } finally {
    Pop-Location
  }
}
