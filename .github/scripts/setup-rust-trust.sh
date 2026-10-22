#!/usr/bin/env bash
set -euo pipefail

bundle="${RUNNER_TEMP}/qir-ca-bundle.pem"
# Bootstrap with the fixed bundle immediately, outside the project's cooldown.
uv run --no-config --no-project --isolated --python python --with certifi==2026.10.19 \
  python -c 'import certifi, shutil, sys; shutil.copyfile(certifi.where(), sys.argv[1])' "$bundle"

export SSL_CERT_FILE="$bundle"
export CURL_CA_BUNDLE="$bundle"
export CARGO_HTTP_CAINFO="$bundle"
export GIT_SSL_CAINFO="$bundle"
for name in SSL_CERT_FILE CURL_CA_BUNDLE CARGO_HTTP_CAINFO GIT_SSL_CAINFO; do
  printf '%s=%s\n' "$name" "${!name}" >> "$GITHUB_ENV"
done

rustup self update
rustup --version
