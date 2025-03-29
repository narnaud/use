# Use - 🚀 Setting up development environment easily

**Use** is a command line tool to setup environment defined in a json file. The syntax is easy enough to be able to handle setup with multiple environment variables and secondary scripts.

- Append, prepend or set new environment variables,
- Add directory to `PATH`,
- Run external scripts,
- Reuse existing environments,
- Change the terminal tab title based on the environment
- Extend the prompt with the environment name

As we can't change the console environment from a binary, use is using a dual strategy:

- a binary, `use-config`, to extract all the information for setting up the environment
- a shell script to setup for a given shell, using the output of `use-config`

## Installation

Install **use** with [scoop](<https://scoop.sh/>):

```
scoop bucket add narnaud https://github.com/narnaud/scoop-bucket
scoop install use
```

## Usage

To use **use**, you need to have `use-config` in the `PATH`, as well as your shell integration setup.

```
Command-line utility to setup environment

Usage: use [OPTIONS] [NAME] [COMMAND]

Commands:
  set   Adjust use's settings
  help  Print this message or the help of the given subcommand(s)

Arguments:
  [NAME]  Name of the environment to use

Options:
  -l, --list     List all environments
  -c, --create   Create a new config file
  -h, --help     Print help
  -V, --version  Print version
```

## Configuration

**Use** expect a YAML configuration file in `~/.useconfig.yaml` (or `%USERPROFILE%\.useconfig.yaml` on Windows). Here is a small example:

```yaml
# Visual Studio
msvc2022:
  display: Microsoft Visual Studio 2022 - x64
  defer:
    - C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat

# Qt, all versions
qt{}:
  display: Qt {} - MSVC - x64
  pattern:
    path: C:\Qt
    regex: "^(\\d+\\.\\d+\\.\\d+)$"
  use:
    - msvc2022
  set:
    QTDIR: C:\Qt\{}\msvc2019_64\
  append:
    CMAKE_PREFIX_PATH: C:\Qt\{}\msvc2019_64\
  path:
    - C:\Qt\{}\msvc2019_64\bin

# Example environment
example:
  display: Name of the configuration
  use:
    - qt6
    - msvc2022
    - other
    - configuration
    - names
  defer:
    - C:\example\path\to\script.bat
    - C:\example\other\path\to\script.bat
  set:
    EXAMPLE_VAR: example value
  append:
    EXAMPLE_VAR_APPEND: value appended to EXAMPLE_VAR_APPEND
  prepend:
    EXAMPLE_VAR_PREPEND: value prepended to EXAMPLE_VAR_PREPEND
  path:
    - C:\example\path\to\add\to\path
    - C:\example\other\path\to\add\to\path
  go: C:\example\path\to\go\to
```

The YAML file is a map of environments, the key being used as the environment name when running the command. For each environment, you can have:

- `display`: string displayed when setting the environment
- `set`: list of environment variables to initialize
- `append`: append values to environment variables
- `prepend`: prepend values to environment variables
- `path`: add paths to the `PATH` environment variable
- `defer`: call one or multiple scripts
- `use`: reuse existing environment (they will be setup before)
- `go`: go to a particular directory at the end of the setup
- `pattern`: use pattern matching to create multiple environments with one definition (see below)

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
    QTDIR: C:\Qt\{}\msvc2019_64\
  append:
    CMAKE_PREFIX_PATH: C:\Qt\{}\msvc2019_64\
  path:
    - C:\Qt\{}\msvc2019_64\bin
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

## Shell integration

All shell integrations expect `use-config` to be accessible in the `PATH`.

### Changing the terminal title

By default, **use** is going to change the terminal title using the environment name. You are free to change the settings:

```batch
use set --update-title false
```

Set it to `true` to go back to the default behavior.

### Cmd (Windows)

A batch file is available here: `<use>/cmd/use.cmd`. Add it to your path or create an alias using `doskey`:

```
doskey use = C:\path\to\<use>\cmd\use.cmd
```

### Clink (Windows)

[Clink](https://chrisant996.github.io/clink/) is a cmd on steroid (you should definitely use it!).

The integration is done by loading a lua script. Add the `<use>/clink` directory to the list of install scripts:

```
clink installscripts C:\path\to\<use>\clink
```

The clink integration has completion (you should really use clink!).

### Powershell

The integration is done via a powershell module, name `use`. Make sure the module is available from the `PSModulePath`, then import it in your powershell profile:

```pwsh
Import-Module posh-use
```

The powershell integration has completion.

### Writing your own integration script

It should be fairly easy to integrate with other shells (contributions are very welcome). The output of `use-config` will be something like that:

```
❯ use-config.exe knut
 Configuring Microsoft Visual Studio 2022 - x64
DEFER: C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat
 Configuring Qt 6.8.2 - MSVC - x64
SET: QTDIR=C:\Qt\6.8.2\msvc2022_64\
SET: CMAKE_PREFIX_PATH=C:\Qt\6.8.2\msvc2022_64\
PATH: C:\Qt\6.8.2\msvc2022_64\bin
 Configuring Knut
GO: C:\dev\knut\knut
SET: USE_PROMPT=knut
TITLE: Knut
    Finished setting up Knut
```

The parsing is quite easy:

- `DEFER: script`: run the script `script`
- `SET: var=value`: set the environment variable `var` to `value`
- `PATH: path`: prepend a path to the `PATH` environment variable
- `GO: path`: go to the directory `path`
- `TITLE: string`: change the console tab title to `string`
- All other lines should be displayed as is.

> Note: as you can see, there are no `APPEND:` or `PREPEND:`: `use-config` automatically change them to `SET:` commands.

With the script, the same command should display:

```
❯ use knut
 Configuring Microsoft Visual Studio 2022 - x64
 Configuring Qt 6.8.2 - MSVC - x64
 Configuring Knut
    Finished setting up Knut
```

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
