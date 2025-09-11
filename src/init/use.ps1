#!/usr/bin/env pwsh

# Create a new dynamic module so we don't pollute the global namespace with our functions and
# variables
$null = New-Module use {

    ## Use fonction -------------------------------------------------------
    function Use-Environment {
        param(
            [Parameter(Mandatory = $false)]
            [string[]]$Args
        )

        # Handle special cases: empty args or commands that should be run directly
        if ($Args.Count -eq 0 -or $Args[0] -in @('init', 'config', 'list', 'set', 'print', 'help') -or $Args[0] -match '^-') {
            & ::USE:: $Args
            return
        }

        # Set an environment
        Invoke-Expression (& ::USE:: $Args | Out-String)
    }

    ## Completer -------------------------------------------------------
    $scriptBlock = {
        param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)

        & ::USE:: list | Where-Object {
            $_ -like "$wordToComplete*"
        } | ForEach-Object {
            "$_"
        }
    }
    Register-ArgumentCompleter -CommandName Use-Environment -ParameterName Args -ScriptBlock $scriptBlock

    ## Exported variables --------------------------------------------------
    $ENV:USE_SHELL = "powershell"

    ## Exported commands and aliases -----------------------------------------------
    Set-Alias use Use-Environment
    Export-ModuleMember Use-Environment
    Export-ModuleMember -Alias use
}
