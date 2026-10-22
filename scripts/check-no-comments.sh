#!/bin/sh
set -eu
cd "$(dirname "$0")/.."

rust_files="$(find src tests gen-contracts -type f -name '*.rs' -not -path '*/target/*' | sort)"
shell_files="$(find scripts tests .githooks -type f \( -name '*.sh' -o -path '.githooks/*' \) | sort)"

rust_findings="$(printf '%s\n' "$rust_files" | LC_ALL=C xargs awk -v quote="'" '
function is_ident(ch) { return ch ~ /[A-Za-z0-9_]/ }
function report() { printf "%s:%d: %s\n", FILENAME, FNR, $0; found = 1 }
function starts_raw_string(line, at,    before, rest) {
  rest = substr(line, at)
  if (rest !~ /^r#*"/) return 0
  before = at > 1 ? substr(line, at - 1, 1) : ""
  if (before == "b") before = at > 2 ? substr(line, at - 2, 1) : ""
  return !is_ident(before)
}
FNR == 1 { in_string = 0; raw_hashes = -1; in_block = 0 }
{
  line = $0
  length_of_line = length(line)
  i = 1
  if (in_block) { report(); if (index(line, "*/") > 0) in_block = 0; next }
  while (i <= length_of_line) {
    ch = substr(line, i, 1)
    following = substr(line, i + 1, 1)
    if (raw_hashes >= 0) {
      closing = "\"" substr("################################", 1, raw_hashes)
      if (substr(line, i, length(closing)) == closing) { i += length(closing); raw_hashes = -1; continue }
      i++
      continue
    }
    if (in_string) {
      if (ch == "\\") { i += 2; continue }
      if (ch == "\"") in_string = 0
      i++
      continue
    }
    if (ch == "/" && (following == "/" || following == "*")) { report(); if (following == "*" && index(substr(line, i + 2), "*/") == 0) in_block = 1; next }
    if (ch == "\"") { in_string = 1; i++; continue }
    if (ch == "r" && starts_raw_string(line, i)) {
      match(substr(line, i + 1), /^#*/)
      raw_hashes = RLENGTH
      i += RLENGTH + 2
      continue
    }
    if (is_ident(ch)) { while (i <= length_of_line && is_ident(substr(line, i, 1))) i++; continue }
    if (ch == quote) {
      if (following == "\\") { closing_at = index(substr(line, i + 3), quote); i += closing_at > 0 ? closing_at + 3 : 1; continue }
      if (substr(line, i + 2, 1) == quote) { i += 3; continue }
      if (following >= "\200") { closing_at = index(substr(line, i + 2), quote); i += closing_at > 0 ? closing_at + 2 : 1; continue }
    }
    i++
  }
}
END { exit found }
' || true)"

shell_findings="$(printf '%s\n' "$shell_files" | xargs awk '
FNR == 1 && /^#!/ { next }
/^[[:space:]]*#/ { printf "%s:%d: %s\n", FILENAME, FNR, $0 }
')"

findings="$(printf '%s\n%s\n' "$rust_findings" "$shell_findings" | sed '/^$/d')"
if [ -n "$findings" ]; then
  printf 'comments are not allowed:\n%s\n' "$findings" >&2
  exit 1
fi
