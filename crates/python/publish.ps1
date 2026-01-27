#!/usr/bin/env pwsh

<#
.SYNOPSIS
    Publish the MiraScript Python package to PyPI
.DESCRIPTION
    Automatically update the version in pyproject.toml, commit changes, build, and publish the Python package
.PARAMETER Version
    The version to release, e.g. 0.1.0 or 0.2.0-beta.1
.PARAMETER NoCommit
    Skip git commit
.PARAMETER NoTag
    Skip creating a git tag
.PARAMETER NoPublish
    Skip publishing to PyPI
.EXAMPLE
    .\publish.ps1 -Version 0.2.0
.EXAMPLE
    .\publish.ps1 -Version 0.2.0-beta.1 -NoPublish
#>

[CmdletBinding()]
param (
    [Parameter(Mandatory = $true)]
    [string] $Version,
    [switch] $NoCommit,
    [switch] $NoTag,
    [switch] $NoPublish
)

& {
    $ErrorActionPreference = "Stop"
    chcp 65001 | Out-Null
    Push-Location $PSScriptRoot

    try {
        # Validate version format
        if ($Version -notmatch '^\d+\.\d+\.\d+(-[a-zA-Z0-9.-]+)?$') {
            throw "Invalid version format: $Version. Expected like: 0.1.0 or 0.1.0-beta.1"
        }

        Write-Host "Preparing to publish MiraScript Python package v$Version" -ForegroundColor Cyan
        Write-Host ""

        # 1. Update version in pyproject.toml
        Write-Host "[1/5] Updating pyproject.toml version..." -ForegroundColor Yellow
        $pyprojectPath = "pyproject.toml"

        if (Test-Path $pyprojectPath) {
            $content = Get-Content $pyprojectPath -Raw

            if ($content -match '(?m)^\s*version\s*=\s*"([^"]+)"') {
                $currentVersion = $Matches[1]
                Write-Host "  Current version: $currentVersion" -ForegroundColor Cyan
            }
            else {
                Write-Host "  Could not detect current version in pyproject.toml" -ForegroundColor Yellow
            }
        }
        else {
            throw "pyproject.toml not found"
        }

        $newContent = $content -replace '(?m)^version\s*=\s*"[^"]*"', "version = `"$Version`""

        if ($null -ne $currentVersion -and $currentVersion -eq $Version) {
            Write-Host "  No change: pyproject.toml already at version $Version" -ForegroundColor Yellow
        }
        else {
            if ($content -eq $newContent) {
                throw "Failed to update version; please check pyproject.toml format"
            }

            $newContent | Out-File $pyprojectPath -NoNewline -Encoding utf8
            Write-Host "  ✓ Version updated to $Version" -ForegroundColor Green
        }

        # 2. Check maturin
        Write-Host "[2/5] Checking for maturin..." -ForegroundColor Yellow
        try {
            $maturinVersion = maturin --version 2>&1
            Write-Host "  ✓ Found maturin: $maturinVersion" -ForegroundColor Green
        }
        catch {
            Write-Host "  ✗ maturin not found, installing..." -ForegroundColor Red
            pip install maturin==1.7.5
        }

        # 3. build the package
        Write-Host "[3/5] Building Python package..." -ForegroundColor Yellow
        maturin build --release
        if ($LASTEXITCODE -ne 0) {
            throw "Build failed"
        }
        Write-Host "  ✓ Build succeeded" -ForegroundColor Green

        # 3.1 use docker image pythonenv to build linux wheel
        Write-Host "[3.1/5] Building Linux wheel using Docker..." -ForegroundColor Yellow
        $dockerImage = "pythonenv:latest"
        Write-Host "  Using Docker image: $dockerImage" -ForegroundColor Cyan
        docker run --rm -v D:\code\core:/core -w /core/crates/python --entrypoint "/bin/sh" $dockerImage -c ". /opt/venv/bin/activate && maturin build --release"
        if ($LASTEXITCODE -ne 0) {
            throw "Linux build failed"
        }
        Write-Host "  ✓ Linux build succeeded" -ForegroundColor Green
        


        # 4. Git commit and tag
        if (-not $NoCommit) {
            Write-Host "[4/5] Committing changes to Git..." -ForegroundColor Yellow
            git add pyproject.toml
            git commit -m "build: chore(python): release v$Version" 
            
            if ($LASTEXITCODE -eq 0) {
                Write-Host "  ✓ Changes committed" -ForegroundColor Green
                
                if (-not $NoTag) {
                    $tagName = "python-v$Version"
                    git tag -a $tagName -m "Python package v$Version"
                    Write-Host "  ✓ Created tag: $tagName" -ForegroundColor Green
                }
            }
            else {
                Write-Host " No changes to commit " -ForegroundColor Yellow
            }
        }
        else {
            Write-Host "[4/5] Skipping Git commit" -ForegroundColor Gray
        }



        

        # 5. publish to PyPI
        if (-not $NoPublish) {
            Write-Host "[5/5] Publishing to PyPI..." -ForegroundColor Yellow
            Write-Host "  Ensure you have configured PyPI token or login credentials" -ForegroundColor Cyan
            
            # 使用 maturin publish 发布
            # 如果需要指定 token，可以使用: maturin publish --username __token__ --password $env:PYPI_TOKEN
            maturin publish 
            docker run --rm -v D:\code\core:/core -w /core/crates/python -v C:\Users\dps-zdm\.pypirc:/root/.pypirc --entrypoint "/bin/sh" $dockerImage -c ". /opt/venv/bin/activate && maturin publish --no-sdist"
            if ($LASTEXITCODE -eq 0) {
                Write-Host "  ✓ Publish succeeded!" -ForegroundColor Green
            }
            else {
                throw "Publish failed"
            }
        }
        else {
            Write-Host "[5/5] Skipping publish to PyPI" -ForegroundColor Gray
        }

        Write-Host ""
        Write-Host " Release process complete!" -ForegroundColor Green
        Write-Host ""
        Write-Host "Next steps:" -ForegroundColor Cyan
        if (-not $NoCommit -and -not $NoTag) {
            Write-Host "  • Push to remote: git push && git push --tags" -ForegroundColor White
        }
        if (-not $NoPublish) {
            Write-Host "  • Verify release: pip install mirascript==$Version" -ForegroundColor White
            Write-Host "  • View package: https://pypi.org/project/mirascript/$Version/" -ForegroundColor White
        }
        Write-Host ""
    }
    catch {
        Write-Host ""
        Write-Host " publish error $_" -ForegroundColor Red
        Write-Host ""
        exit 1
    }
    finally {
        Pop-Location
    }
}
