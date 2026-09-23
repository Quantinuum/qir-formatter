# QIR Formatter

`qir-formatter` is a Rust-backed Python library for rendering execution results into the
[QIR labeled output schema](https://github.com/qir-alliance/qir-spec/blob/2.1/specification/output_schemas/Labeled.md).
It accepts shot-oriented result data in the internal `USER:<TYPE>:<TAG>` form
and emits the text format expected by tools that consume labeled QIR output.

## Installation

```sh
pip install qir-formatter
```

The release workflow builds native wheels for standard (GIL-enabled) CPython
3.10 and newer on:

- Linux x86-64 and ARM64 (manylinux2014, glibc 2.17 or newer)
- macOS Intel and Apple Silicon
- Windows x86-64

Installing a matching wheel does not require Rust. Other targets, including
Alpine/musl, Windows ARM64, and 32-bit platforms, do not have prebuilt wheels;
installation falls back to the source distribution and requires Rust 1.85 or
newer and a platform C/C++ build toolchain. Source builds on these targets are
not tested by the release workflow. Free-threaded Python and alternative Python
implementations are not covered by the wheel matrix.

Every wheel is installed and checked with the Python compatibility tests before
publishing.

## Usage

The formatter operates on a list of shots, where each shot is a list of
`(name, value)` tuples. User-facing values should use the
`USER:<TYPE>:<TAG>` naming convention.

```python
from qir_formatter import QirLabeledFormatter, QsysShots

results: QsysShots = [
    [
        ("USER:INT:shots", 42),
        ("USER:BOOL:accepted", 1),
        ("USER:RESULT_ARRAY:bits", [1, 0, 1]),
    ]
]

attributes = {
    "qir_profiles": "base_profile",
    "required_num_qubits": "3",
    "required_num_results": "3",
}

output = QirLabeledFormatter().qir_labeled_output(results, attributes)
print(output)
```

This produces:

```text
HEADER	schema_id	labeled
HEADER	schema_version	2.1
START
METADATA	entry_point
METADATA	qir_profiles	base_profile
METADATA	required_num_qubits	3
METADATA	required_num_results	3
OUTPUT	INT	42	shots
OUTPUT	BOOL	true	accepted
OUTPUT	RESULT_ARRAY	101	bits
END	0
```

Only `USER` records are emitted. Known raw types currently map to the labeled
QIR schema as follows:

- `INT` and `UINT` become `INT`
- `FLOAT` becomes `DOUBLE`
- `BOOL` becomes `BOOL`
- `RESULT` becomes `RESULT`
- `RESULT_ARRAY` becomes `RESULT_ARRAY`
- `QIRARRAY` becomes `ARRAY`
- `QIRTUPLE` becomes `TUPLE`

Malformed values are skipped rather than raising, which makes the formatter
safe to use on partially clean result streams. Integer records use the signed
64-bit domain defined by the QIR record-output ABI. Python integers outside
`-(2**63)` through `2**63 - 1` are malformed and are skipped with a warning.

## Rust API

`src/labeled_formatter.rs` is the single formatting implementation. It exposes
the formatter's validation, header/footer, value, shot, and complete-output
methods with typed Rust inputs. Writer methods append to a concrete `QirOutput`
buffer containing text and a malformed-value count. Floats use Rust's standard
formatting, not Python's exact decimal notation.

Consumers can rename the package dependency to give the library an idiomatic
crate name:

```toml
[dependencies]
qir_formatter = { package = "qir-formatter", git = "https://github.com/Quantinuum/qir-formatter" }
```

```rust
use qir_formatter::{
    QShotValType, QirLabeledFormatter, QirMetadata, QsysShotItemValue, QsysShots,
};

let results: QsysShots = vec![vec![(
    "USER:INT:answer".into(),
    QsysShotItemValue::Scalar(QShotValType::Int(42)),
)]];
let output = QirLabeledFormatter::new().qir_labeled_output(&results, &QirMetadata::new());
assert!(output.contains("OUTPUT\tINT\t42\tanswer\n"));
```

Malformed-value warnings use Rust's `log` crate with debug-formatted context.
The calling application configures the logger. The Python adapter also emits
the original plain warning through Python's `logging` module.

`src/python.rs` only adapts Python inputs, return values, text writers, and
logging. It does not build QIR records or dispatch through Python formatter
methods. The unchanged Python tests define the compatibility target; Python
subclass/override hooks and weak references are not supported. Output is buffered
before being passed to a Python writer.

The core has no Python dependency and can be tested independently:

```sh
cargo test
```

## Development

The primary contributor workflow uses `uv` and a Rust toolchain. The package is
built as a PyO3 extension with Maturin; its public Python imports are unchanged.

```sh
uv sync --all-groups
```

Optional local environment files such as `devenv.*` and `.envrc` are kept for
maintainer convenience, but they are not required to build or test the project.

Pull requests should keep changes focused, include tests when behavior changes,
and use the conventional-commit title format already enforced by the repo, for
example `fix: handle malformed result arrays` or `docs: expand README usage
example`.

### Linting

```sh
uv run ruff format --check src tests
uv run ruff check src tests
uv run ty check src tests
```

### Testing

```sh
cargo test
uv run pytest
```

The Python compatibility suite in `tests/` is a permanent release gate for the
public bindings and verifies them against the original Python behavior. It
complements the Rust unit tests and should remain in CI. Both suites share the
fixtures in `src/tests/data/` and run in CI. After editing Rust source, rebuild
the extension with `uv sync --all-groups --reinstall-package qir-formatter`
before running pytest.

### Dependency Audit

```sh
uv audit --locked --preview-features audit
```

This scans pinned dependencies directly from `uv.lock`. The repo also
configures `uv` with a 7-day dependency cooldown so routine resolution avoids
newly uploaded packages while the ecosystem has time to surface supply-chain
issues.

### Git Hooks

The repo includes a `prek` configuration in `.pre-commit-config.yaml`.

```sh
uvx prek install
uvx prek run --all-files
```

This runs `ruff` formatting and lint fixes, a `ty` type-check pass, and a small
set of builtin file hygiene checks before commit.

## Support

For bug reports, feature requests, or questions about the public package, open
an issue in the
[GitHub repository](https://github.com/Quantinuum/qir-formatter/issues).
