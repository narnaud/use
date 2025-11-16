# Use - 🚀 Setting up development environment easily

**Use** is a command line tool to setup environment defined in a json file. The syntax is easy enough to be able to handle setup with multiple environment variables and secondary scripts.

- Append, prepend or set new environment variables,
- Add directory to `PATH`,
- Run external scripts,
- Reuse existing environments,
- Change the terminal tab title based on the environment
- Extend the prompt with the environment name

As we can't change the console environment from a binary, use is using a dual strategy:

- a binary, `use`, to extract all the information for setting up the environment
- a shell script to setup for a given shell, using the output of `use`

## Installation

### Installation via [Scoop](https://scoop.sh/) (preferred)

Install **use** with [scoop](<https://scoop.sh/>):

```
scoop bucket add narnaud https://github.com/narnaud/scoop-bucket
scoop install use
```

### Or via archive files

1. Go to the [Releases](https://github.com/narnaud/use/releases) page
2. Download the latest `use-x86_64-pc-windows-msvc.zip` file
3. Extract the files from it into a directory.

## Set up your shell

### Powershell

```powershell
Invoke-Expression (&use init powershell)
```

### Clink

> [!TIP]
> If you install **use** with scoop, you don't need to do anything, it will install such a script automatically.

You need [clink](https://chrisant996.github.io/clink/) wit Cmd. Create a file at this path %LocalAppData%\clink\use.lua with the following contents:

```cmd
load(io.popen('use init cmd'):read("*a"))()
```

## Usage

Once **use** is initialized in your shell, you can type `use` in your shell to have a list of known nvironment, or `use --help` for the help.

```
Command-line utility to setup environment

Usage: use [NAME] [COMMAND]

Commands:
  init  Prints the shell function used for shell integration
  list  List all environments
  set   Adjust use's settings
  help  Print this message or the help of the given subcommand(s)

Arguments:
  [NAME]  Name of the environment to use

Options:
  -h, --help     Print help
  -V, --version  Print version
```

**use** is using a yaml configuration file to defines the different environments, see below.

## Configuration

**Use** expect a YAML configuration file in `~/.config/use.yaml` (or `%USERPROFILE%\.config\use.yaml` on Windows). Here is a small example:

```yaml
# Example environment
example:
  display: Name of the configuration
  use:
    - qt6
    - msvc2022
    - other
    - configuration
    - names
  set:
    EXAMPLE_VAR: example value
    EXAMPLE_VAR_OTHER: other value
  append:
    EXAMPLE_VAR_APPEND: value appended to EXAMPLE_VAR_APPEND
    EXAMPLE_VAR_OTHER_APPEND: value appended to EXAMPLE_VAR_OTHER_APPEND
  prepend:
    EXAMPLE_VAR_PREPEND: value prepended to EXAMPLE_VAR_PREPEND
    EXAMPLE_VAR_OTHER_PREPEND: value prepended to EXAMPLE_VAR_OTHER_PREPEND
  path:
    - C:\example\path\to\add\to\path
    - C:\example\other\path\to\add\to\path
  script: |
    echo "Something"
    echo "Something else"
  go: C:\example\path\to\go\to

# Visual Studio 2022 x64
msvc2022:
  display: Microsoft Visual Studio 2022 - x64
  for_cmd:
    script: |
      call "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvarsall.bat" amd64 > nul
  for_powershell:
    script: |
      & "C:\Program Files\Microsoft Visual Studio\2022\Professional\Common7\Tools\Launch-VsDevShell.ps1" -SkipAutomaticLocation -Arch amd64 *>$null

# Qt, all versions, using pattern
qt{}:
  display: Qt {} - MSVC - x64
  pattern:
    path: C:\Qt
    regex: "^(\\d+\\.\\d+\\.\\d+)$"
  use:
    - msvc2022
  set:
    QTDIR: C:\Qt\{}\msvc2022_64\
  append:
    CMAKE_PREFIX_PATH: ${QTDIR}
  path:
    - ${QTDIR}\bin
```

The YAML file is a map of environments, the key being used as the environment name when running the command. For each environment, you can have:

- `display`: string displayed when setting the environment
- `use`: reuse existing environment (they will be setup before)
- `set`: list of environment variables to initialize
- `append`: append values to environment variables
- `prepend`: prepend values to environment variables
- `path`: add paths to the `PATH` environment variable
- `script`: raw lines to call as a script
- `go`: go to a particular directory at the end of the setup
- `pattern`: use pattern matching to create multiple environments with one definition (see below)

### Shell specific values

It is possible to have for some shell some specific values for one environment, for example:

```yaml
msvc2022:
  display: Microsoft Visual Studio 2022 - x64
  for_cmd:
    script: |
      call "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvarsall.bat" amd64 > nul
  for_powershell:
    script: |
      & "C:\Program Files\Microsoft Visual Studio\2022\Professional\Common7\Tools\Launch-VsDevShell.ps1" -SkipAutomaticLocation -Arch amd64 *>$null
```

For the `msvc2022` environment, the display is shared, but the script is different for cmd and powershell:

- `for_cmd`: values specific to the cmd shell,
- `for_powershell` (or `for_pwsh`): values specific to the powershell shell.

You can change almost everything, except `pttern`:

- `display`, `script` and `go` are replaced,
- `use`, `set`, `append`, `prepend`, `path` are extended.

### Pattern matching

It is possible to define multiple environments in a single definition. This is especially interesting if you have multiple versions of the same lib or app.

For example, here is a definition for Qt on Windows (the `{}` are replaced with the regex capture):

```yaml
qt{}:
  display: Qt {} - MSVC - x64
  pattern:
    path: C:\Qt
    regex: "^(\\d+\\.\\d+\\.\\d+)$"
  use:
    - msvc2022
  set:
    QTDIR: C:\Qt\{}\msvc2022_64\
  append:
    CMAKE_PREFIX_PATH: ${QTDIR}
  path:
    - ${QTDIR}\bin
```

The interesting part is the `pattern` key:

- `path`: gives the path to look at
- `regex`: the regex to match files/dirs in the path

So if I have installed for example Qt 5.12.2, Qt 6.5.3 and Qt 6.8.2, I should have those directories under C:\Qt:

```cmd
C:Qt
+- 5.12.2
+- 6.5.3
+- 6.8.2
```

And it will create 3 different environments: `qt5.12.2`, `qt6.5.3` and `qt6.8.2`.

You can use partial key to use an environment. Following on the same example:

- `use qt`: set up the latest Qt version, here Qt 6.8.2
- `use qt6.5`: set up the latest Qt 6.5 version available, here 6.5.3
- `use qt5.12.2`: set up an explicit Qt version

It works the same for the YAML configuration, you can use partial keys:

```yaml
example:
  use:
    - qt6
```

### Environement variables

It's possible to use environment variables as part of the value of a field. The syntax for that is `${ENV_VARIABLE}`.
It will be replaced by the current shell way of handling environement variables, for example `%ENV_VARIABLE%` on cmd, and `$env:ENV_VARIABLE` on powershell.

For example, the `qt` environment reuse `QTDIR` in other variables:

```yaml
  set:
    QTDIR: C:\Qt\{}\msvc2022_64\
  append:
    CMAKE_PREFIX_PATH: ${QTDIR}
  path:
    - ${QTDIR}\bin
```

It's important to note the order in which the different values are set, and there are no particular orders inside set, append and prepend.

1. set
2. append
3. prepend
4. path
5. script
6. go

Please note that an algorithm tries to resolve dependencies between environment variables if needed. For example, if you have that:

```yaml
set:
  KEY1: $(KEY2)
  KEY2: foo
```

**use** will ensure that `KEY2` is set before `KEY1`.

## Shell integration

### Changing the terminal title

By default, **use** is going to change the terminal title using the environment name. You are free to change the settings:

```batch
use set --update-title false
```

Set it to `true` to go back to the default behavior.

### Writing your own integration script

It should be fairly easy to integrate with other shells (contributions are very welcome).
To do that, you need:

1. initialization script (in init directory): integration of use in the Shell, goal is to provide a method `use` that read and execute the output of the `use` executable,
2. shell printer (int `shell` directory): provide the shell specific way to set/append/prepend variables, change the PATH and change the title.

Please check the powershell and clink integration.

## Prompt integration

**Use** can integrate with any existing prompt, as it sets an environment variable `USE_PROMPT` with the name of the environment in use.

### Oh-my-posh

If you are using [Oh My Posh](https://ohmyposh.dev/), add a new segment like that:

```
{
    "type": "text",
    "template": " {{.Env.USE_PROMPT}} "
},
```

### Clink Flex Prompt

If you are using [clink](https://chrisant996.github.io/clink/) and the [clink-flex-prompt](https://github.com/chrisant996/clink-flex-prompt), you can add a segment to the `flexprompt.settings.left_prompt` or `flexprompt.settings.right_prompt` like this:

```
flexprompt.settings.right_prompt = "{env:label=󱁤:var=USE_PROMPT:color=black}"
```
