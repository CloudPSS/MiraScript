#!/usr/bin/env pwsh

[CmdletBinding()]
param (
    [switch] $NoCommit,
    [switch] $NoTag
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

    Write-Host "Publishing version $Version@$NpmTag" -ForegroundColor Yellow

    # set cargo version for all packages
    foreach ($file in Get-ChildItem -File ./crates/*/Cargo.toml) {
        $dirname = Split-Path $file.FullName -Parent | Split-Path -Leaf
        Write-Output "Updating crates/$dirname to version $Version"
        $lines = Get-Content $file.FullName
        $lines = $lines -replace '^version\s*=\s*".*?"$', "version = `"$Version`""
        $content = "$($lines -join "`n")`n"
        Set-Content $file.FullName -Value $content -NoNewLine
        git add $file.FullName
    }
    cargo update --offline
    git add ./Cargo.lock

    # set npm version for all packages
    foreach ($file in Get-ChildItem -File ./packages/*/package.json) {
        $dirname = Split-Path $file.FullName -Parent | Split-Path -Leaf
        Write-Output "Updating packages/$dirname to version $Version"
        $pkg = Read-Json $file.FullName
        $pkg.version = $Version
        Write-Json $pkg $file.FullName
        git add $file.FullName
    }

    # set root npm version
    git add ./package.json

    if (-not $NoCommit) {
        git commit -m "v$Version"

        if (-not $NoTag) {
            git tag -a "v$Version" -m "v$Version"
        }
    }

    Pop-Location
}
