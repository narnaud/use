## Commands --------------------------------------------------------------------
<#
.SYNOPSIS
Configures the environment based on the output of the `use-config` command.

.DESCRIPTION
The `Use-Environment` function processes the output of the `use-config` command and performs various actions based on the directives in the output.
It supports setting environment variables, modifying the PATH, changing the current directory, updating the console window title, and executing deferred commands.

.PARAMETER Args
An optional array of strings to pass as arguments to the `use-config` command.

.EXAMPLE
Use-Environment dev test
# This example runs the `use-config` command with the arguments "dev" and "test" and applies the resulting environment configuration.

.EXAMPLE
Use-Environment
# This example runs the `use-config` command without any arguments which lists the known environments.

.NOTES
- The function processes specific directives in the output of `use-config`:
    - `SET: <key>=<value>`: Sets an environment variable.
    - `PATH: <path>`: Prepends the specified path to the system PATH.
    - `GO: <path>`: Changes the current working directory to the specified path.
    - `TITLE: <title>`: Updates the console window title.
    - `DEFER: <command>`: Executes a deferred command. If the command is a `.cmd` or `.bat` file, it merges the child process environment variables into the parent process.

- The function writes any unrecognized output from `use-config` to the console.

#>
function Use-Environment {
    param(
        [Parameter(Mandatory = $false)]
        [string[]]$Args
    )

    & use-config $Args | ForEach-Object {
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

    & use-config | Where-Object {
        $_ -like "$wordToComplete*"
    } | ForEach-Object {
        "$_"
    }
}
Register-ArgumentCompleter -CommandName Use-Environment -ParameterName Args -ScriptBlock $scriptBlock

## Exported commands and aliases -----------------------------------------------
Set-Alias use Use-Environment
Export-ModuleMember Use-Environment
Export-ModuleMember -Alias use
