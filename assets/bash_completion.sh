#!/bin/bash

_qad() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local prev="${COMP_WORDS[COMP_CWORD - 1]}"

  if [ "$COMP_CWORD" == 1 ]; then
    if [ "$cur" == "c" ]; then
      # We deliberately chose to complete ci for "c"
      # At best, it won't block the input
      # At worst, it will only be a backspace to correct
      mapfile -t COMPREPLY < <(compgen -W "ci" -- "${cur}")
      return 0
    fi
    mapfile -t COMPREPLY < <(compgen -W "autocomplete ci config list --version --help" -- "${cur}")
    return 0
  fi
  COMPREPLY=()
  if [ "$COMP_CWORD" == 2 ]; then
    if [ "$prev" == "config" ]; then
      if [ "$cur" == "m" ]; then
        mapfile -t COMPREPLY < <(compgen -W "migrate" -- "${cur}")
        return 0
      fi
      mapfile -t COMPREPLY < <(compgen -W "migrate --help" -- "${cur}")
      return 0
    fi
    if [ "$prev" == "ci" ]; then
      mapfile -t jobs < <(qad list)
      mapfile -t COMPREPLY < <(compgen -W "${jobs[*]}" -- "${cur}")
      return 0
    fi
    return 0
  fi
  if [ "$COMP_CWORD" == 3 ]; then
    if [ "${COMP_WORDS[1]}" == "config" ] && [ "${COMP_WORDS[2]}" == "migrate" ]; then
      mapfile -t versions < <(qad config migrate --help | awk ' $1 ~ /./ { print $1 }' | sed '1,/Commands:/ d')
      mapfile -t COMPREPLY < <(compgen -W "$(printf "%s " "${versions[@]}")--help" -- "${cur}")
      return 0
    fi
    return 0
  fi
  return 0
}

complete -F _qad qad
