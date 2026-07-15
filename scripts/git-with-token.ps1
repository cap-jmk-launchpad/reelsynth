# Delegates to the shared reeldemo.io git auth helper (reads GH_TOKEN from .env.local).
param(
    [string]$EnvFile = "C:\Users\Julian\Documents\Programming\reeldemo.io\.env.local",
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$CommandArgs
)

$sharedHelper = "C:\Users\Julian\Documents\Programming\reeldemo.io\scripts\git-with-token.ps1"
if (-not (Test-Path -LiteralPath $sharedHelper)) {
    throw "Shared helper not found: $sharedHelper"
}

& $sharedHelper -EnvFile $EnvFile @CommandArgs
exit $LASTEXITCODE
