# QIR Formatter

`qir-formatter` is a Python library for rendering execution results into the
[QIR labeled output schema](https://github.com/qir-alliance/qir-spec/blob/a177eee5885d99e965f5cb919c8d70b0ab7a6a15/specification/output_schemas/Labeled.md).
It accepts shot-oriented result data in the internal `USER:<TYPE>:<TAG>` form
and emits the text format expected by tools that consume labeled QIR output.

## Installation

```sh
pip install qir-formatter
```

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
safe to use on partially clean result streams.

## Development

The primary contributor workflow uses `uv`.

```sh
uv sync --all-groups
```

Optional local environment files such as `devenv.*` and `.envrc` are kept for
maintainer convenience, but they are not required to build or test the project.

### Linting

```sh
uv run ruff format --check src
uv run ruff check src
uv run ty check src
```

### Testing

```sh
uv run pytest
```

### Dependency Audit

```sh
uv run src/scripts/audit_dependencies.sh
```

The audit uses `uv audit --locked` to scan pinned dependencies directly from
`uv.lock`. The repo also configures `uv` with a 7-day dependency cooldown so
routine resolution avoids newly uploaded packages while the ecosystem has time
to surface supply-chain issues.

## Support

For bug reports, feature requests, or questions about the public package, open
an issue in the
[GitHub repository](https://github.com/Quantinuum/qir-formatter/issues).
