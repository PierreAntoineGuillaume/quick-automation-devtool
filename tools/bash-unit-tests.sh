#!/usr/bin/env bash

set -euo pipefail

base_dirname=$(realpath "$(dirname "$0")/..")

cd "$base_dirname"

source tools/lib/bash-completion-unit-framework.sh # autocomplete_test
source assets/dt_bash_competion.sh                 # _dt

err_counter=0

unit() {
  if ! autocomplete_test "_dt" "$@"; then
    ((err_counter += 1))
  fi
}

unit "dt " "autocomplete" "ci" "config" "-q" "--quiet" "--version" "--help"
unit "dt az" ""
unit "dt a" "autocomplete"
unit "dt auto" "autocomplete"
unit "dt c" "ci"
unit "dt ci " ""
unit "dt co" "config"
unit "dt config " "migrate" "--help"
unit "dt config m" "migrate"
unit "dt config migrate 0.y " "to-toml" "to-yaml --help"

exit "$err_counter"
