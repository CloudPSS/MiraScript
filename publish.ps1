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

    Push-Location $PSScriptRoot
    yarn version --no-git-tag-version

    function Write-Json ($obj, $file) {
        $(ConvertTo-Json -Depth 100 $obj).Replace("`r`n", "`n") + "`n" | Out-File $file -NoNewline
    }
    function Read-Json ($file) {
        return Get-Content $file -Raw | ConvertFrom-Json
    }

    $rootPkg = Read-Json ./package.json
    $Version = $rootPkg.version
    $NpmTag = if ($Version -match "-") { "next" } else { "latest" }

    Write-Information "Publishing version $Version@$NpmTag"

    # set the shared Cargo package version and exact internal dependency pins
    Write-Output "Updating Rust workspace to version $Version"
    $cargoFile = Resolve-Path ./Cargo.toml
    $cargoContent = Get-Content $cargoFile -Raw
    $cargoContent = $cargoContent -replace '(?m)^version\s*=\s*".*?"$', "version = `"$Version`""
    $cargoContent = $cargoContent -replace 'version\s*=\s*"=.*?"', "version = `"=$Version`""
    Set-Content $cargoFile -Value $cargoContent -NoNewLine
    git add $cargoFile

    $metadata = cargo metadata --format-version 1 --no-deps | ConvertFrom-Json
    $wrongVersions = $metadata.packages | Where-Object {
        $_.source -eq $null -and $_.version -ne $Version
    }
    if ($wrongVersions) {
        $details = $wrongVersions | ForEach-Object { "$($_.name)@$($_.version)" }
        throw "Cargo package versions are out of sync: $($details -join ', ')"
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
