# Contributing

## Setup

This project uses `uv` for dependency management and local commands.

```sh
uv sync --all-groups
```

Optional local environment helpers such as `devenv.*` and `.envrc` are for
maintainer convenience. They are not required for normal contribution work.

## Checks

Run the full local validation set before opening or updating a pull request.

```sh
uv run pytest
uv run ruff format --check src
uv run ruff check src
uv run ty check src
```

To run the dependency audit locally:

```sh
uv run src/scripts/audit_dependencies.sh
```

## Pull Requests

Keep changes focused and include tests when behavior changes.

Pull request titles must follow the conventional commits format enforced by the
repository, for example:

```text
fix: handle malformed result arrays
docs: expand README usage example
```

If a change is user-visible, update the relevant documentation in the same pull
request.
