@echo off
::: Note: we can't use setlocal as we want to change the env variables

:::============================================================================
::: Call the use_env executable to get all the env variables
:::============================================================================

for /f "delims=" %%a in ('%~dp0\..\..\target\debug\use_env %*') do (
    call :handle_line "%%a"
)
goto :eof


:::============================================================================
::: Handle all the different lines from the output of use_env
:::============================================================================
:handle_line
set var=%1

echo %1 | findstr /C:"SET:">nul && (
    ::set "%var:~6,-1%"
    goto :end_handle_line
)
echo %1 | findstr /C:"DEFER:">nul && (
    ::call "%var:~8,-1%" >NUL
    goto :end_handle_line
)
echo %1 | findstr /C:"PATH:">nul && (
    ::set "PATH=%var:~7,-1%;%PATH%"
    goto :end_handle_line
)
echo %1 | findstr /C:"GO:">nul && (
    ::cd "%var:~5,-1%"
    goto :end_handle_line
)
echo %var:~1,-1%
:end_handle_line
exit /b 0


:::============================================================================
::: End of file
:::============================================================================
:eof
