#!/usr/bin/env bash

set -Eeuo pipefail
set -o xtrace
DIR=$(dirname "$(realpath "$0")")
SRC_DIR="${DIR}/../qir_formatter"


uv run ty check "${SRC_DIR}" "${DIR}/../tests"

if [[ "${CI:-}" == "true" ]]; then
    uv run ruff format --check "${SRC_DIR}" "${DIR}/../tests"
    uv run ruff check "${SRC_DIR}" "${DIR}/../tests"
else
    uv run ruff format "${SRC_DIR}" "${DIR}/../tests"
    uv run ruff check --fix "${SRC_DIR}" "${DIR}/../tests"
fi
grep "print(" "${SRC_DIR}" --include="*.py" --recursive --files-with-matches && (echo "Found print statement"; exit 1)

echo "Done"
