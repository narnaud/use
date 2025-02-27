@echo off
::: Note: we can't use setlocal as we want to change the env variables

:::============================================================================
::: CHeck if we have a direct call
:::============================================================================
if /i "%1"=="" (
    goto direct_call
)
set arg=%1
if /i "%arg:~0,1%"=="-" (
	goto direct_call
)

:::============================================================================
::: Call the use executable to get all the env variables
:::============================================================================
for /f "delims=" %%a in ('use-config %*') do (
    call :handle_line "%%a"
)
goto :eof

:::============================================================================
::: Direct call to the use executable
:::============================================================================
:direct_call
%~dp0\..\use %*
goto :eof

:::============================================================================
::: Handle all the different lines from the output of use
:::============================================================================
:handle_line
set var=%1

echo %1 | findstr /C:"SET:">nul && (
    set "%var:~6,-1%"
    goto :end_handle_line
)
echo %1 | findstr /C:"DEFER:">nul && (
    call "%var:~8,-1%" >NUL
    goto :end_handle_line
)
echo %1 | findstr /C:"PATH:">nul && (
    set "PATH=%var:~7,-1%;%PATH%"
    goto :end_handle_line
)
echo %1 | findstr /C:"GO:">nul && (
    cd "%var:~5,-1%"
    goto :end_handle_line
)
echo %1 | findstr /C:"TITLE:">nul && (
    TITLE %var:~8,-1%
    goto :end_handle_line
)
echo %var:~1,-1%
:end_handle_line
exit /b 0


:::============================================================================
::: End of file
:::============================================================================
:eof
