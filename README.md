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

## Usage

To use **use**, you need to have `use-config` in the `PATH`, as well as your shell integration setup.

```
Usage: use [OPTIONS] [NAME]

Arguments:
  [NAME]  Name of the environment to use

Options:
  -l, --list     List all environments
  -c, --create   Create a new config file
  -h, --help     Print help
  -V, --version  Print version
```

## Configuration

**Use** expect a configuration file in `~/.useconfig.json` (or `%USERPROFILE%\.useconfig.json` on Windows). Here is a small example:

```json
{
    "msvc2022": {
        "display": "MVSC 2022",
        "defer": [
            "C:\\Program Files\\Microsoft Visual Studio\\2022\\Professional\\VC\\Auxiliary\\Build\\vcvars64.bat"
        ]
    },
    "qt6.7": {
        "display": "Qt 6.7.0",
        "set": {
            "QTDIR": "C:\\Qt\\6.7.0\\msvc2019_64\\"
        },
        "append": {
            "CMAKE_PREFIX_PATH": "C:\\Qt\\6.7.0\\msvc2019_64\\"
        },
        "path": [
            "C:\\Qt\\6.7.0\\msvc2019_64\\bin"
        ]
    },
    "knut": {
        "display": "Knut",
        "use": [
            "msvc2022",
            "qt6.7"
        ],
        "go": "C:\\dev\\knut\\knut"
    }
}
```

The json file is a map of environments, the key being used as the environment name when running the command. For each environment, you can have:

- `display`: string displayed when setting the environment
- `set`: list of environment variables to initialize
- `append`: append values to environment variables
- `prepend`: prepend values to environment variables
- `path`: add paths to the `PATH` environment variable
- `defer`: call one or multiple scripts
- `use`: reuse existing environment (they will be setup before)
- `go`: go to a particular directory at the end of the setup

## Shell integration

All shell integrations expect `use-config` to be accessible in the `PATH`.

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

The clink integration also have completion by default (you should really use clink!).

### Writing your own integration script

It should be fairly easy to integrate with other shells (copntribution are very welcome). The output of `use-config` will be something like that:

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
