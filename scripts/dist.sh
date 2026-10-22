#!/bin/sh
set -eu

if [ "$#" -lt 1 ]; then
  echo "usage: scripts/dist.sh <out-dir> [target-triple...]" >&2
  exit 2
fi

daemon_root="$(cd "$(dirname "$0")/.." && pwd)"
mkdir -p "$1"
out_dir="$(cd "$1" && pwd)"
shift
if [ "$#" -eq 0 ]; then
  set -- "$(rustc -vV | sed -n 's/^host: //p')"
fi

sha256_of() {
  if command -v sha256sum > /dev/null 2>&1; then
    sha256sum "$1" | cut -d ' ' -f 1
  else
    shasum -a 256 "$1" | cut -d ' ' -f 1
  fi
}

for target in "$@"; do
  (cd "$daemon_root" && cargo build --release --target "$target")
  binary="$out_dir/mootbox-$target"
  cp "$daemon_root/target/$target/release/mootbox" "$binary"
  printf '%s' "$(sha256_of "$binary" | tr 'A-F' 'a-f')" > "$binary.sha256"
  echo "$binary"
done
