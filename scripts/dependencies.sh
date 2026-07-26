#!/usr/bin/env bash

set -euo pipefail

if [ "$#" -lt 1 ]; then
  exit 1
fi

version="$1"
target_arch="${2}"
install_dir="${3:-/usr/local/bin}"

if [ -z "$target_arch" ]; then
  case "$(uname -m)" in
    x86_64) target_arch="amd64" ;;
    aarch64) target_arch="arm64" ;;
    *)
      echo "Unable to infer target architecture from uname -m"
      exit 1
      ;;
  esac
fi

case "$target_arch" in
  amd64) trunk_arch="x86_64-unknown-linux-gnu" ;;
  arm64) trunk_arch="aarch64-unknown-linux-gnu" ;;
  *)
    echo "Unsupported target architecture for trunk: $target_arch"
    exit 1
    ;;
esac

archive="trunk-${trunk_arch}.tar.gz"
archive_sha="${archive}.sha256"
base_url="https://github.com/trunk-rs/trunk/releases/download/v${version}"
tmp_dir="$(mktemp -d)"

trap 'rm -rf "$tmp_dir"' EXIT

curl -fsSL -o "$tmp_dir/$archive" "$base_url/$archive"
curl -fsSL -o "$tmp_dir/$archive_sha" "$base_url/$archive_sha"

# (
#   cd "$tmp_dir"
#   sha256sum -c "$archive_sha"
# )

tar -xzf "$tmp_dir/$archive" -C "$install_dir" trunk
chmod +x "$install_dir/trunk"
