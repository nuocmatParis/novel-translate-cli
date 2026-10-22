# Run the current source; Go keeps the compiled executable in its temporary cache.
$ErrorActionPreference = 'Stop'

$repoDirectory = Split-Path -Parent $PSScriptRoot
$goCommand = Get-Command go -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
if ($null -eq $goCommand) {
    throw 'Go 1.25.5+ is required. Install Go and reopen the terminal to run from source.'
}

$appArguments = @($args)
if ($appArguments.Count -eq 0) {
    $appArguments = @('tui')
}

$previousGoWork = $env:GOWORK
Push-Location -LiteralPath $repoDirectory
try {
    $env:GOWORK = 'off'
    & $goCommand.Source run -trimpath '-ldflags=-s -w' ./cmd/novel-translate-cli @appArguments
    $appExitCode = $LASTEXITCODE
}
finally {
    $env:GOWORK = $previousGoWork
    Pop-Location
}
exit $appExitCode
