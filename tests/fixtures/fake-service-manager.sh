#!/bin/sh
here="$(dirname "$0")"
printf '%s\n' "$*" >> "$here/calls"
if [ -f "$here/failing" ]; then
  failing="$(cat "$here/failing")"
  case " $* " in *" $failing "*) echo "$failing refused" >&2; exit 5;; esac
fi
if [ -f "$here/stopped" ]; then
  case "$*" in *is-active*) exit 3 ;; esac
fi
if [ -f "$here/output" ]; then
  cat "$here/output"
fi
