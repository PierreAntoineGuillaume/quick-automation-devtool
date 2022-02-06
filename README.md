# dev-tool

dev-tool `dt` is a cli tool to help with local developpement.

Prepare a dev-tool file `dt.toml` in your working directory, then just run `dt ci`

## Format of the dev-tool file dt.toml

As of Feb the 6th, 2022 we are in alpha mode.

Current format version is "0.x"

Expected syntax is:

```toml
# dt.toml
version = "0.x"
[jobs]
jobname = ["instruction --option", "instruction2 arg1 arg2"]
job2name = ["instruction3"]
```

## Communication channel

You can join the slack channel by clicking [here](https://join.slack.com/t/devtool-for-auto-ci/shared_invite/zt-12wd1k6w7-6PnQBAAyrnvoo60tovV3Gw).
