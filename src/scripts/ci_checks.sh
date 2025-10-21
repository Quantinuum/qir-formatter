#!/usr/bin/env bash

set -Eeuo pipefail
set -o xtrace
DIR=$(dirname "$(realpath "$0")")
SRC_DIR="${DIR}/../qir_formatter"


uvx pyright "${SRC_DIR}" "${DIR}/../tests"
uvx mypy "${SRC_DIR}" "${DIR}/../tests" --namespace-packages

RUFF_FORMAT_ARGS=()
RUFF_CHECK_ARGS=()
if [[ "${CI:-}" == "true" ]]; then
    RUFF_FORMAT_ARGS+=(--check)
else
    RUFF_CHECK_ARGS+=(--fix)
fi

uvx ruff format "${RUFF_FORMAT_ARGS[@]}" "${SRC_DIR}" "${DIR}/../tests"
uvx ruff check "${RUFF_CHECK_ARGS[@]}" "${SRC_DIR}" "${DIR}/../tests"
grep "print(" "${SRC_DIR}" --include="*.py" --recursive --files-with-matches && (echo "Found print statement"; exit 1)

echo "Done"
