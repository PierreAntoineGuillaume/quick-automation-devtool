#!/usr/bin/env bash

set -euo pipefail

base_dirname=$(realpath "$(dirname "$0")/..")

cd "$base_dirname"

source tools/lib/bash-completion-unit-framework.sh # autocomplete_test
source assets/bash_completion.sh                   # _qad

err_counter=0

unit() {
  if ! autocomplete_test "_qad" "$@"; then
    ((err_counter += 1))
  fi
}

unit "qad " "app" "autocomplete" "ci" "debug" "list" "has-ci" "--version" "--help" "--file" "--no-tty"
unit "qad az" ""
unit "qad a" "app"
unit "qad auto" "autocomplete"
unit "qad c" "ci"

exit "$err_counter"
