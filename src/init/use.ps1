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

        # Handle special cases: empty args, 'init', 'list', or 'set'
        if ($Args.Count -eq 0 -or $Args[0] -in @('init', 'list', 'set', 'help') -or $Args[0] -match '^-') {
            & ::USE:: $Args
            return
        }

        # Set an environment
        & ::USE:: $Args | ForEach-Object {
            if ($_ -match '^SET:\s*(\w+)=(.+)$') {
                $key = $matches[1]
                $value = $matches[2]
                [Environment]::SetEnvironmentVariable($key, $value)
            }
            elseif ($_ -match '^PATH:\s*(.+)$') {
                $path = $matches[1]
                $env:path = $path + ';' + $env:path
            }
            elseif ($_ -match '^GO:\s*(.+)$') {
                $path = $matches[1]
                Set-Location $path
            }
            elseif ($_ -match '^TITLE:\s*(.+)$') {
                $title = $matches[1]
                $Host.UI.RawUI.WindowTitle = $title
            }
            elseif ($_ -match '^DEFER:\s*(.+)$') {
                $cmd = $matches[1]
                if ($cmd -match '\.cmd$|\.bat$') {
                    # Based on https://github.com/KierDugan/PsEnv

                    $command = ''
                    if ($cmd -match " ") {
                        $command += "`"$cmd`" "
                    }
                    else {
                        $command += "$cmd "
                    }

                    # Use the normal command prompt to execute the command
                    cmd /c "$command & set" | foreach {
                        # Look for every variable line and merge the child environment
                        # into the parent
                        if ($_ -match '=') {
                            $key, $value = $_.Split('=')
                            [Environment]::SetEnvironmentVariable($key, $value)
                        }
                    }
                }
                else {
                    & $cmd
                }
            }
            else {
                Write-Output $_
            }
        }
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
