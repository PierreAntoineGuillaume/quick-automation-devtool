#!/bin/bash

_dt() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  #  local prev="${COMP_WORDS[COMP_CWORD-1]}"

  if [ "$COMP_CWORD" == 1 ]; then
    mapfile -t COMPREPLY < <(compgen -W "autocomplete ci" -- "${cur}")
    return 0
  fi
  COMPREPLY=()
  return 0
}

complete -F _dt dt
