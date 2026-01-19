#!/usr/bin/env pwsh

[CmdletBinding()]
param (
    [switch] $NoCommit,
    [switch] $NoTag,
    [switch] $NoPublish
) 
& {
    $ErrorActionPreference = "Stop"
    $env:COREPACK_ENABLE_STRICT = 0
    $env:SKIP_YARN_COREPACK_CHECK = 1
    yarn version --no-git-tag-version

    function Write-Json ($obj, $file) {
        $(ConvertTo-Json -Depth 100 $obj).Replace("`r`n", "`n") + "`n" | Out-File $file -NoNewline
    }
    function Read-Json ($file) {
        return Get-Content $file -Raw | ConvertFrom-Json
    }

    Push-Location $PSScriptRoot

    $rootPkg = Read-Json ./package.json
    $Version = $rootPkg.version
    $NpmTag = if ($Version -match "-") { "next" } else { "latest" }

    $packages = Get-ChildItem ./packages -Directory
    $packageNames = $packages | ForEach-Object {
        Push-Location $_
        $pkg = Read-Json ./package.json
        Pop-Location
        $pkg.name
    }

    Write-Host "Building version $Version" -ForegroundColor Yellow
    pnpm -r build

    Write-Host "Publishing version $Version@$NpmTag" -ForegroundColor Yellow
    pnpm -r --workspace-concurrency=1 exec pnpm version "$Version" --no-git-tag-version --no-workspaces-update
    pnpm -r --workspace-concurrency=1 exec git add ./package.json

    git add ./package.json

    if (-not $NoCommit) {
        git commit -m "v$Version"

        if (-not $NoTag) {
            git tag -a "v$Version" -m "v$Version"
        }
    }

    if (-not $NoPublish) {
        npm login --registry https://registry.npmjs.org
        pnpm -r publish --access public --registry https://registry.npmjs.org --tag $NpmTag
    }

    if (-not $NoPublish) {
        Remove-Item Alias:curl -ErrorAction SilentlyContinue
        $packageNames | ForEach-Object {
            Write-Host "Syncing $_" -ForegroundColor Yellow
            $result = curl -X PUT "https://registry-direct.npmmirror.com/$_/sync" 2>$null | ConvertFrom-Json
            if ($result.ok -eq $false) {
                Write-Error $result.error $result.reason
            } else {
                Write-Host "OK, see https://registry-direct.npmmirror.com/-/package/$_/syncs/$($result.logId)/log"
            }
        }
    }

    Pop-Location
}
