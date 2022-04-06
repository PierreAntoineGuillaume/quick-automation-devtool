#!/usr/bin/env bash

# This is a lib for testing bash autocompletion
# It works by mimicking behavior of what complete should do when tab is pressed
# given I could not make compgen work

assert() {
  if [ "${3+set}" = "set" ]; then
    assertionMessage=$3
  else
    assertionMessage="%s != %s\n"
  fi

  successMessage=${4:-}

  if test "$1" = "$2"; then
    # shellcheck disable=SC2059
    printf "$successMessage" "$1"
  else
    code=$?
    # shellcheck disable=SC2059
    printf >&2 "$assertionMessage" "$1" "$2"
    return "$code"
  fi
}

parse_string_for_completion() {
  COMP_LINE=$1
  read -r -a COMP_WORDS <<<"$COMP_LINE"

  # COMP_CWORD must be -1 if the last character is not in the IFS
  # we're using a sentinel value "word" to concatenate with string
  # so that if the string ends with a space, it will add a full word to the string
  # and in all cases, remove one word from the total
  # https://en.wikipedia.org/wiki/Sentinel_value
  read -r -a count_word_helper <<<"${COMP_LINE}word"
  COMP_CWORD="${#count_word_helper[@]}"
  ((COMP_CWORD -= 1))
}

autocomplete_test() {
  exec=$1
  line=$2
  parse_string_for_completion "$line"
  shift 2
  expected=$*
  local oldstate
  oldstate=$(set +o | grep nounset | grep -o -- '[-+]o')
  set +o nounset
  $exec
  set "$oldstate" nounset
  assert "$expected" "${COMPREPLY[*]}" "$(printf '❌ %-10s expected: "%%s" actual: "%%s"' "$line")\n" "$(printf '✅ %-10s expected: "%%s"' "$line")\n"
}
