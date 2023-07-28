#!/bin/bash

_qad() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local prev="${COMP_WORDS[COMP_CWORD - 1]}"

  if [ "$COMP_CWORD" == 1 ]; then
    if [ "$cur" == "a" ]; then
      # We deliberately chose to complete ci for "c"
      # At best, it won't block the input
      # At worst, it will only be a backspace to correct
      mapfile -t COMPREPLY < <(compgen -W "app" -- "${cur}")
      return 0
    fi
    mapfile -t COMPREPLY < <(compgen -W "app autocomplete ci debug list has-ci --version --help --file --no-tty" -- "${cur}")
    return 0
  fi
  COMPREPLY=()
  if [ "$COMP_CWORD" == 2 ]; then
    if [ "$prev" == "ci" ] || [ "$prev" == "debug" ]; then
      if compgen -G "qad.y*ml*" > /dev/null; then
        mapfile -t jobs < <(qad list 2>/dev/null)
      else
        jobs=()
      fi
      mapfile -t COMPREPLY < <(compgen -W "--help ${jobs[*]}" -- "${cur}")
      return 0
    fi
    if [ "$prev" == -f ] || [ "$prev" == "--file" ]; then
      # todo find how to
      :
    fi
    return 0
  fi
  if ! [[ "${COMP_WORDS[*]}" =~ "--help" ]]; then
    mapfile -t COMPREPLY < <(compgen -W "--help" -- "${cur}")
  fi
  return 0
}

complete -F _qad qad
