#!/usr/bin/env bash

set -Eeuo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
ARTIFACT_DIR=$(mktemp -d "${TMPDIR:-/tmp}/qir-formatter-audit.XXXXXX")

REQUIREMENTS_FILE=$(mktemp "${ARTIFACT_DIR}/pip-audit-requirements.XXXXXX.txt")
trap 'rm -rf "${ARTIFACT_DIR}"' EXIT

uv export \
    --project "${ROOT_DIR}" \
    --frozen \
    --all-groups \
    --output-file "${REQUIREMENTS_FILE}" \
    >/dev/null

audit_args=(
    --progress-spinner
    off
    --requirement
    "${REQUIREMENTS_FILE}"
    --require-hashes
    --disable-pip
)

if [[ -n "${PIP_AUDIT_IGNORE_VULNS:-}" ]]; then
    # Accept a whitespace-delimited list of vulnerability IDs to ignore.
    read -r -a ignored_vulns <<< "${PIP_AUDIT_IGNORE_VULNS}"
    for vuln_id in "${ignored_vulns[@]}"; do
        audit_args+=(--ignore-vuln "${vuln_id}")
    done
fi

uv --project "${ROOT_DIR}" run pip-audit "${audit_args[@]}"
