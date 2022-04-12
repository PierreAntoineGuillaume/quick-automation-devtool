# Quick Automation Dev-tool

quick automation dev-tool `qad` is a CLI tool to help with local developpement.

Prepare a `qad.yaml` file in your working directory, then just run `qad ci`

The configuration filename should match the perl regex: `/qad.ya?ml(.dist)?/`.

A single configuration file will be parsed. `qad.yaml` is prioritary over `qad.yaml.dist`

A `version` key pilots the format of the configuration to avoid breaking changes.

More details on [the current configuration version file](docs/VERSION_1.md).

![A cute quad drawing](qad.png)
