#!/usr/bin/env bash
# Render the Homebrew formula for a published release.
#
#   packaging/homebrew/render-formula.sh 0.2.0 > ../homebrew-tap/Formula/surveyor.rb
#
# Reads SHA256SUMS from the v<version> GitHub release and fills surveyor.rb.in.
set -euo pipefail

version="${1:?usage: render-formula.sh <version>}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
sums_url="https://github.com/andrewdean/surveyor/releases/download/v${version}/SHA256SUMS"

sums="$(curl -fsSL "${sums_url}")"

sha_for() {
  local target="$1"
  local sha
  sha="$(printf '%s\n' "${sums}" | awk -v f="surveyor-${version}-${target}.tar.gz" '$2 == f { print $1 }')"
  if [[ -z "${sha}" ]]; then
    echo "no checksum for ${target} in ${sums_url}" >&2
    exit 1
  fi
  printf '%s' "${sha}"
}

sed \
  -e "s/@VERSION@/${version}/g" \
  -e "s/@SHA_AARCH64_APPLE_DARWIN@/$(sha_for aarch64-apple-darwin)/" \
  -e "s/@SHA_X86_64_APPLE_DARWIN@/$(sha_for x86_64-apple-darwin)/" \
  -e "s/@SHA_X86_64_UNKNOWN_LINUX_GNU@/$(sha_for x86_64-unknown-linux-gnu)/" \
  "${here}/surveyor.rb.in"
