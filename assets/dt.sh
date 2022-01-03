_dt() {
  local cur
  COMPREPLY=()
  cur="${COMP_WORDS[COMP_CWORD]}"
  COMPREPLY=($(compgen -W "ci autocomplete" -- "${cur}"))
  return 0
}

complete -F _dt dt
