
stubs:
	cd qir-formatter-core && cargo run --bin stub_gen

reinstall:
	uv sync --reinstall-package qir_formatter_core

python: stubs reinstall
