param(
    [string]$EnvFile = "C:\Users\Julian\Documents\Programming\reeldemo.io\.env.local",
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Args
)

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
& "$scriptDir\git-with-token.ps1" -EnvFile $EnvFile git push @Args
exit $LASTEXITCODE
