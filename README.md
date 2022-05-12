# Quick Automation Dev-tool

quick automation dev-tool `qad` is a CLI tool to help with local developpement.

Prepare a `qad.yaml` file in your working directory, then just run `qad ci`

The configuration filename should match the perl regex: `/qad.ya?ml(.dist)?/`.

A single configuration file will be parsed. `qad.yaml` is prioritary over `qad.yaml.dist`

A `version` key pilots the format of the configuration to avoid breaking changes.

More details on [the current configuration version file](docs/VERSION_1.md).

![A cute quad drawing](docs/qad.png)

# Major commands

```shell
$ qad --help
Usage: qad [--version] [<command>] [<args>]

A tool to help with testing, and dev-related tasks

Options:
  --version         show the executable version
  --help            display usage information

Commands:
  ci                play the ci
  list              list jobs
  autocomplete      generate bash completion script
  config            interract with configuration

$ qad ci --help 
Usage: qad ci [<nested>]

play the ci

Positional Arguments:
  nested            an optionnal job or group to run

Options:
  --help            display usage information

```

# Examples

```shell
# play the whole process
$ qad ci

# play only the fmt job
$ qad ci fmt

# play the run gorup
$ qad ci group:run
```

## Autocompletion

`qad` has a built-in bash-completion script ; you can see it when you run `qad autocomplete`.

`qad` tells you what to run to register it.

```shell
$ qad autocomplete 
# To register qad's bash autocompletion script
# put the following content including the shebang (#!/bin/bash) in
# ~/.local/share/bash-completion/completions/qad:
# mkdir -p ~/.local/share/bash-completion/completions
# qad autocomplete > ~/.local/share/bash-completion/completions/qad
#!/bin/bash

_qad() {
...

$ mkdir ~/.local/share/bash-completion/completions/
$ qad autocomplete > ~/.local/share/bash-completion/completions/qad
$ exit

```

I advise to rely on the autocompletion to use `qad`
