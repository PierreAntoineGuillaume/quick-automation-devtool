# Version 1

The first configuration file has a single required field: `version`

Setting `version: 1` tells the parser to use _this_ configuration. 

```qad.yaml
version: 1
jobs:
  fmt:
    script: [ cargo fmt "$RUST_CHANGED_FILES" ]
    skip_if: test -z $RUST_CHANGED_FILES
  clippy:
    script: [ cargo clippy --tests --color always -- -D warnings ]
    image: rust:latest
  tests:
    script: [ cargo test --color always ]
    image: rust:latest

groups:
  - fmt
  - run

display:
  mode: sequence|summary
  ok: "\e[32m✔\e[0m"
  ko: "\e[31m✕\e[0m"
  cancelled: "\e[35m✕\e[0m"
  spinner:
    frames: [ "⣇", "⣦", "⣴", "⣸", "⢹", "⠻", "⠟", "⡏" ]
    per_frames: 70
env: |
  set -euo pipefail
  CURRENT_BRANCH=$(git branch --show-current)
  LAST_COMMON_COMMIT=$(git merge-base "main" "$(git branch --show-current)")
  CHANGED_FILES=$(cat <(git diff --name-only --diff-filter=RAM $LAST_COMMON_COMMIT) <(git ls-files -o --exclude-standard))
  RUST_CHANGED_FILES="$(awk '$1 ~ /.rs/ { print $1 }' <<<$CHANGED_FILES)"
```

Version 1 has 5 sections:

- jobs
- groups
- contraints
- display
- env

# jobs

**jobs** is a `map<job_name, job>` where `job_name` is a string.

## job

**job** is an object:

**script** is an array of strings, the scripts to be executed. These scripts are parsed using `$SHELL`

**image** is an optionnal field, telling qad to run the job in docker. The keys described in the `env` section are forwarded to used container

**group** is an optionnal field, telling qad how to schedule the job.

**skip_if** is an optionnal field, telling qad when to skip the job. Job will be skipped if `$SHELL -c string` exits with 0.

# groups

**groups** is one of the two ways to schedule jobs.

All jobs bound to group will be run when _the group_ is being run.

Only a single _group_ can be run at once.

_groups_ are run consequently.

If a job within a group fails, all jobs of next groups will be _cancelled_.

# contraints

Contraints is the other way to schedule jobs.

Jobs can block other jobs.

## blocks

**blocks** is a `Map<job_name, array<job_name>>`, it ensures the first `job_name` will be necessary to run the following ones.

## needs

**needs** is a `Map<job_name, array<job_name>>`, it ensures all first `job_name` will be dependant of the following ones.

# display

**mode** selects the display mode of the running ci. It has two possible values: sequence or summary

**ok** is the expression used to represent sucessful jobs.

**ko** is the expression used to represent failed jobs.

**cancelled** is the expression used to represent cancelled jobs.

**spinner** is the sequence of spinner frames and the refresh rate used

```yaml
frames: [". ", ".. ", "...", ".. ", ".  "]
per_frame: 70
```
# env

env is a string parsed with `$SHELL`, and each `key=` will be forwarded to the jobs.

# extra files

extra_files is used to import **jobs** from another qad file. It expects a list of strings
