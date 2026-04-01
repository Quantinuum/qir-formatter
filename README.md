# QIR Formatter

QIR Formatter is a Python library for formatting results from the
[QIR Labeled Output Schema](https://github.com/qir-alliance/qir-spec/blob/a67da3a3bd902ba7cd94a98722ca5f8c89f5a81a/specification/under_development/output_schemas/Labeled.md).

## Development

```sh
uv sync --all-groups
```

## Linting

```sh
uv run ruff format --check src
uv run ruff check src
uv run ty check src
```

## Testing

```sh
uv run pytest
```
