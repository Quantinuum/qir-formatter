#!/usr/bin/env bash

set -Eeuo pipefail
set -o xtrace
DIR=$(dirname "$(realpath "$0")")
REPO_ROOT=$(realpath "${DIR}/../..")
SRC_DIR="${DIR}/../qir_formatter"
STUB_ROOT="${DIR}/.."
STUB_PATH="src/qir_formatter/_native.pyi"
TEST_DIR="${DIR}/../../tests"


uv run maturin generate-stubs --locked --out "${STUB_ROOT}"
if [[ "${CI:-}" == "true" ]]; then
    STUB_STATUS=$(git -C "${REPO_ROOT}" status --porcelain --untracked-files=all -- "${STUB_PATH}")
    if [[ -n "${STUB_STATUS}" ]]; then
        echo "Generated stub is stale:"
        echo "${STUB_STATUS}"
        exit 1
    fi
fi
uv run ty check "${SRC_DIR}" "${TEST_DIR}"
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
uv run pytest "${TEST_DIR}"

if [[ "${CI:-}" == "true" ]]; then
    uv run ruff format --check "${SRC_DIR}" "${TEST_DIR}"
    uv run ruff check "${SRC_DIR}" "${TEST_DIR}"
else
    uv run ruff format "${SRC_DIR}" "${TEST_DIR}"
    uv run ruff check --fix "${SRC_DIR}" "${TEST_DIR}"
fi
if grep -El '(print|println|eprint|eprintln|dbg)[[:space:]]*!' "${DIR}/.." --include="*.rs" --recursive --files-with-matches; then
    echo "Found Rust print or dbg macro"
    exit 1
fi

echo "Done"
