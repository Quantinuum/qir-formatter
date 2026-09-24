"""Public package surface tests."""

import ast
import inspect
from io import StringIO
from pathlib import Path

from qir_formatter import QirLabeledFormatter, QsysShots


def test_top_level_exports_support_basic_usage() -> None:
    """The package should expose a stable top-level import surface."""
    results: QsysShots = [[("USER:INT:answer", 42)]]

    output = QirLabeledFormatter().qir_labeled_output(results, {})

    assert "OUTPUT\tINT\t42\tanswer\n" in output


def test_top_level_formatter_can_emit_values() -> None:
    """The main formatter should be directly importable from the package root."""
    out = StringIO()

    QirLabeledFormatter().emit(out, "RESULT_ARRAY", "bits", [1, 0, 1])

    assert out.getvalue() == "OUTPUT\tRESULT_ARRAY\t101\tbits\n"


def test_formatter_stub_and_runtime_docstrings_match() -> None:
    """Editor documentation and runtime help should expose the same docstrings."""
    stub = Path(__file__).resolve().parents[1] / "src/qir_formatter/_native.pyi"
    module = ast.parse(stub.read_text())
    formatter = next(
        node
        for node in module.body
        if isinstance(node, ast.ClassDef) and node.name == "QirLabeledFormatter"
    )
    assert ast.get_docstring(formatter) == inspect.getdoc(QirLabeledFormatter)
    for method in formatter.body:
        if isinstance(method, ast.FunctionDef):
            docstring = ast.get_docstring(method)
            assert docstring, f"Missing stub docstring for {method.name}"
            assert docstring == inspect.getdoc(
                getattr(QirLabeledFormatter, method.name)
            ), method.name
