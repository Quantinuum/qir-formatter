# Python compatibility tests

These are the original pytest cases from commit `9d29324`, before the Rust
migration. Assertions and inputs are unchanged. The only test-code changes
resolve fixtures relative to the repository root instead of the
working directory. The tests reuse the four fixtures in `src/tests/data/`,
which are also used by the Rust tests.

From the repository root, with Rust available (e.g. in the development shell):

```sh
uv sync
uv run pytest
```

Pytest is included in the project's `test` dependency group. These tests run
against the installed Rust extension and are included in CI alongside the Rust
tests. To rebuild the extension after editing Rust source, run
`uv sync --all-groups --reinstall-package qir-formatter` before testing.

To see individual cases, run it with `-v`.

Passing these tests establishes compatibility for the original suite's covered
behavior, not an exhaustive proof for every possible input.
