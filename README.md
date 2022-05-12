# Quick Automation Dev-tool

quick automation dev-tool `qad` is a CLI tool to help with local developpement.

Prepare a `qad.yaml` file in your working directory, then just run `qad ci`

The configuration filename should match the perl regex: `/qad.ya?ml(.dist)?/`.

A single configuration file will be parsed. `qad.yaml` is prioritary over `qad.yaml.dist`

A `version` key pilots the format of the configuration to avoid breaking changes.

More details on [the current configuration version file](docs/VERSION_1.md).

![A cute quad drawing](qad.png)

# Major commands

```shell
$ qad --help
Usage: qad [--version] [<command>] [<args>]

A tool to help with testing, and dev-related tasks

Options:
  --version         show the executable version
  --help            display usage information

Commands:
  ci [nested]               play the ci
  list              list jobs
  autocomplete      generate bash completion script
  config            interract with configuration
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
