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
    if [ "$cur" == "a" ]; then
      # We deliberately chose to complete ci for "c"
      # At best, it won't block the input
      # At worst, it will only be a backspace to correct
      mapfile -t COMPREPLY < <(compgen -W "app" -- "${cur}")
      return 0
    fi
    mapfile -t COMPREPLY < <(compgen -W "app autocomplete ci config list has-ci --version --help --file" -- "${cur}")
    return 0
  fi
  COMPREPLY=()
  if [ "$COMP_CWORD" == 2 ]; then
    if [ "$prev" == "config" ]; then
      mapfile -t COMPREPLY < <(compgen -W "migrate --help" -- "${cur}")
      return 0
    fi
    if [ "$prev" == "ci" ]; then
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
  if [ "$COMP_CWORD" == 3 ]; then
    if [ "${COMP_WORDS[1]}" == "config" ] && [ "${COMP_WORDS[2]}" == "migrate" ]; then
      mapfile -t versions < <(qad config migrate --help | awk ' $1 ~ /./ { print $1 }' | sed '1,/Commands:/ d')
      mapfile -t COMPREPLY < <(compgen -W "--help $(printf "%s " "${versions[@]}")" -- "${cur}")
      return 0
    fi
  fi
  if ! [[ "${COMP_WORDS[*]}" =~ "--help" ]]; then
    mapfile -t COMPREPLY < <(compgen -W "--help" -- "${cur}")
  fi
  return 0
}

complete -F _qad qad
